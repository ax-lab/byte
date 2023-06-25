use super::*;

/// Manages a list of input sources for the compiler.
pub struct SourceList {
	data: Arc<SourceListData>,
}

impl SourceList {
	/// Returns a new source list rooted at the given path.
	pub fn new<P: AsRef<Path>>(base_path: P) -> Result<Self> {
		let base_path = std::fs::canonicalize(base_path.as_ref())?;
		let data = SourceListData {
			base_path: base_path,
			sources: Default::default(),
			sources_by_path: Default::default(),
		};
		Ok(Self { data: data.into() })
	}

	/// Add a source file text to the list. Returns a new unique [`Span`].
	///
	/// Note that the source name is informative and only used for compiler
	/// messages.
	pub fn add_text<T: Into<String>, U: Into<Vec<u8>>>(&self, name: T, data: U) -> Span {
		let name = name.into();
		let data = data.into();
		let mut sources = self.data.sources.write().unwrap();
		let offset = Self::compute_next_source_offset(&sources);
		let length = data.len();
		let source = SourceData { offset, name, data };
		sources.push(source);
		Span {
			handle: self.handle(),
			offset,
			length,
		}
	}

	/// Add a source file to the list, if it has not been added. Returns a
	/// unique [`Span`] for the given file.
	///
	/// Files are uniquely mapped by their canonical path. Multiple calls
	/// for the same file will return the same [`Span`].
	pub fn add_file<T: AsRef<Path>>(&self, path: T) -> Result<Span> {
		let path = path.as_ref();
		let full_path = if path.is_relative() {
			self.data.base_path.join(path)
		} else {
			path.to_owned()
		};

		let full_path = match std::fs::canonicalize(full_path) {
			Ok(path) => path,
			Err(err) => {
				let path = path.to_string_lossy();
				let base = self.data.base_path.to_string_lossy();
				return Err(Errors::from(format!(
					"could not solve `{path}`: {err}\n    -- base path is `{base}`"
				)));
			}
		};

		let mut by_path = match self.data.sources_by_path.write() {
			Ok(data) => data,
			Err(data) => data.into_inner(),
		};

		if let Some(index) = by_path.get(&full_path) {
			let source = match self.data.sources.read() {
				Ok(data) => data,
				Err(data) => data.into_inner(),
			};
			let source = &source[*index];
			Ok(Span {
				handle: self.handle(),
				offset: source.offset,
				length: source.data.len(),
			})
		} else {
			let name = path.to_string_lossy().to_string();
			let data = match std::fs::read(path) {
				Ok(data) => data,
				Err(err) => {
					let path = full_path.to_string_lossy();
					return Err(Errors::from(format!("opening `{path}` ({err}")));
				}
			};
			let mut sources = self.data.sources.write().unwrap();
			by_path.insert(full_path, sources.len());

			let offset = Self::compute_next_source_offset(&sources);
			let length = data.len();
			let source = SourceData { offset, name, data };
			sources.push(source);

			Ok(Span {
				handle: self.handle(),
				offset,
				length,
			})
		}
	}

	fn handle(&self) -> Handle<SourceListData> {
		Handle::new(&self.data)
	}

	fn compute_next_source_offset(sources: &[SourceData]) -> usize {
		sources
			.last()
			.map(|x| {
				let end = x.offset + x.data.len();
				// ensure input sources do not overlap by padding the offset
				end + (Self::OFFSET_ROUNDING - end % Self::OFFSET_ROUNDING)
			})
			.unwrap_or(0)
	}

	// Pad the next input offset rounding to a multiple of this.
	const OFFSET_ROUNDING: usize = 64;
}

//====================================================================================================================//
// Span
//====================================================================================================================//

/// Represents a range of source text from a [`SourceList`].
#[derive(Clone, Eq, PartialEq)]
pub struct Span {
	handle: Handle<SourceListData>,
	offset: usize,
	length: usize,
}

impl Span {
	/// Create a new cursor at the start of the span.
	pub fn start(&self) -> Cursor {
		let source = self.handle.get().to_inner();
		Cursor {
			span: self.clone(),
			source,
			line: 0,
			column: 0,
			indent: 0,
			tab_width: 0,
		}
	}

	/// Length of the span in bytes.
	pub fn len(&self) -> usize {
		self.length
	}

	/// Globally unique offset for the start position of this span across all
	/// source code.
	pub fn offset(&self) -> usize {
		self.offset
	}

	/// Raw data for this span.
	pub fn data(&self) -> HandleMap<SourceListData, [u8]> {
		self.source_data().map(|source| {
			let sta = self.offset - source.offset;
			let end = sta + self.length;
			let data = &source.data[sta..end] as *const [u8];
			unsafe { &*data }
		})
	}

	/// Span for the full source text, if this is a partial span. Otherwise
	/// returns the current span itself.
	pub fn source(&self) -> Span {
		let source = self.source_data();
		Span {
			handle: self.handle.clone(),
			offset: source.offset,
			length: source.data.len(),
		}
	}

	/// Text for this span.
	pub fn text(&self) -> HandleMap<SourceListData, str> {
		self.data().map(|data| std::str::from_utf8(data).unwrap())
	}

	/// Name for the source of this span.
	pub fn source_name(&self) -> HandleMap<SourceListData, str> {
		self.source_data().map(|source| source.name.as_str())
	}

	fn source_data(&self) -> HandleMap<SourceListData, SourceData> {
		self.handle.get_map(|data| {
			let sources = match data.sources.read() {
				Ok(data) => data,
				Err(data) => data.into_inner(),
			};
			let index = sources.partition_point(|x| x.offset + x.data.len() < self.offset);
			let source = &sources[index] as *const SourceData;
			unsafe { &*source }
		})
	}
}

impl Debug for Span {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		let name = self.source_name();
		let text = self.text();

		let source = self.source_data();
		let sta = self.offset - source.offset;
		let end = sta + self.length;
		let len = self.length;

		let full = sta == 0 && self.length == source.data.len();

		write!(f, "<Span ")?;
		if text.len() <= 50 {
			write!(f, "{text:?}")?;
		} else {
			write!(f, "{len} bytes")?;
		}
		write!(f, " @{name}")?;
		if !full {
			write!(f, "[{sta}…{end}]")?
		}
		write!(f, ">")
	}
}

//====================================================================================================================//
// Cursor
//====================================================================================================================//

/// Indexes a position in a [`Span`] and provides methods for reading its text.
#[derive(Clone)]
pub struct Cursor {
	#[allow(unused)]
	source: Arc<SourceListData>, // ensure source references are valid for the lifetime of the cursor
	span: Span,
	line: usize,
	column: usize,
	indent: usize,
	tab_width: usize,
}

impl Cursor {
	pub fn span(&self) -> &Span {
		&self.span
	}

	/// Make a new copy of the current cursor but with the given line and
	/// column coordinates.
	pub fn with_pos(&self, line: usize, column: usize, indent: usize) -> Self {
		let mut cursor = self.clone();
		cursor.line = line;
		cursor.column = column;
		cursor.indent = indent;
		cursor
	}

	/// Make a new copy of the current cursor setting a new tab-width value.
	///
	/// If the tab-width is zero, then use [`DEFAULT_TAB_WIDTH`].
	pub fn with_tab_width(&self, tab_width: usize) -> Self {
		let mut cursor = self.clone();
		cursor.tab_width = tab_width;
		cursor
	}

	/// Tab-width for the cursor.
	pub fn tab_width(&self) -> usize {
		if self.tab_width == 0 {
			DEFAULT_TAB_WIDTH
		} else {
			self.tab_width
		}
	}

	/// True if at the end of the input.
	pub fn at_end(&self) -> bool {
		self.data().len() == 0
	}

	/// Remaining input data from the cursor position.
	pub fn data(&self) -> &[u8] {
		unsafe { &*self.span.data().as_ptr() }
	}

	/// Remaining input text from the cursor position.
	pub fn text(&self) -> &str {
		unsafe { &*self.span.text().as_ptr() }
	}

	/// Relative line position for the cursor.
	pub fn line(&self) -> usize {
		self.line
	}

	/// Column position for the cursor. Zero is the start of the line.
	///
	/// Note that the column position considers the tab width.
	pub fn column(&self) -> usize {
		self.column
	}

	/// Indent value for the cursor.
	///
	/// This is the total column width for the leading space for the line
	/// at the current position.
	///
	/// At the start of the line and if the current position is at the leading
	/// indentation, then this is the same as the column value.
	pub fn indent(&self) -> usize {
		self.indent
	}

	/// True if the current position is at the leading indentation of the line.
	pub fn is_indent(&self) -> bool {
		self.column == self.indent
	}

	/// Globally unique offset for the current position across all source code.
	pub fn offset(&self) -> usize {
		self.span.offset()
	}

	/// Read the next character in the input and advance the cursor.
	pub fn read(&mut self) -> Option<char> {
		if let Some((char, size)) = self.next_char() {
			self.advance(size);
			Some(char)
		} else {
			None
		}
	}

	/// Read the next character in the input if it is the given character.
	pub fn read_if(&mut self, expected: char) -> bool {
		if let Some((next, size)) = self.next_char() {
			if next == expected {
				self.advance(size);
				true
			} else {
				false
			}
		} else {
			false
		}
	}

	/// Return the next character in the input without advancing the cursor.
	pub fn next_char(&self) -> Option<(char, usize)> {
		let data = self.data();
		if data.len() > 0 {
			let size = char_size(data);
			let char = &data[..size];
			let char = std::str::from_utf8(char)
				.ok()
				.and_then(|x| x.chars().next())
				.unwrap_or('\u{FFFD}');
			Some((char, size))
		} else {
			None
		}
	}

	/// Advance the cursor.
	pub fn advance(&mut self, length: usize) {
		let tab_width = self.tab_width();
		let data = self.data();

		let mut skip = &data[..length];
		let mut line = self.line;
		let mut column = self.column;
		let mut indent = self.indent;
		while skip.len() > 0 {
			let size = if let Some((char, size)) = check_space(skip) {
				let is_indent = column == indent;
				if char == '\t' {
					column += tab_width - (self.column % tab_width);
				} else {
					column += 1;
				}
				if is_indent {
					indent = column;
				}
				size
			} else if let Some(size) = check_line_break(skip) {
				column = 0;
				indent = 0;
				line += 1;
				size
			} else {
				column += 1;
				char_size(skip)
			};
			assert!(size > 0);
			skip = &skip[size..];
		}
		self.line = line;
		self.column = column;
		self.indent = indent;
		self.span.offset += length;
		self.span.length -= length;
	}

	/// Return a new [`Span`] from the given [`Cursor`] to the current position.
	///
	/// Both cursors MUST be from the same input source.
	pub fn span_from(&self, start: &Cursor) -> Span {
		assert!(
			self.offset() >= start.offset()
				&& self.span().source_data().as_ptr() == start.span().source_data().as_ptr()
		);
		let mut span = start.span().clone();
		span.length = self.offset() - start.offset();
		span
	}
}

//====================================================================================================================//
// Internals
//====================================================================================================================//

#[doc(hidden)]
pub struct SourceListData {
	base_path: PathBuf,
	sources: RwLock<Vec<SourceData>>,
	sources_by_path: RwLock<HashMap<PathBuf, usize>>,
}

struct SourceData {
	offset: usize, // global offset for this data in the SourceList
	name: String,  // source name (e.g. file name)
	data: Vec<u8>, // source data
}

//====================================================================================================================//
// Tests
//====================================================================================================================//

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn input_list() -> Result<()> {
		let list = SourceList::new(".")?;
		let a = list.add_text("input A", "123456");
		let b = list.add_text("input B", "some data");

		assert_eq!(a.len(), 6);
		assert_eq!(a.offset(), 0);
		assert_eq!(b.offset(), SourceList::OFFSET_ROUNDING);

		assert_eq!(a.source_name(), "input A");
		assert_eq!(b.source_name(), "input B");
		assert_eq!(a.text(), "123456");
		assert_eq!(b.text(), "some data");

		let c = list.add_file("testdata/input.txt")?;
		assert_eq!(c.offset(), SourceList::OFFSET_ROUNDING * 2);
		assert!(c.source_name().contains("input.txt"));
		assert_eq!(c.text(), "some test data\n");

		assert!(a != b);
		assert!(a != c);
		assert!(b != c);

		let c1 = list.add_file("./testdata/../testdata/input.txt")?;
		assert!(c1 == c);
		assert!(c1.source() == c);
		assert_eq!(c1.text(), c.text());
		assert_eq!(c1.source().text(), c.text());

		Ok(())
	}

	#[test]
	fn cursors() -> Result<()> {
		let list = SourceList::new(".")?;
		let input = list.add_text("input A", "123456");

		let mut cursor = input.start();
		assert_eq!(cursor.read(), Some('1'));
		assert_eq!(cursor.read(), Some('2'));
		assert_eq!(cursor.read(), Some('3'));

		let pos = cursor.clone();

		assert_eq!(cursor.read(), Some('4'));
		assert_eq!(cursor.read_if('!'), false);
		assert_eq!(cursor.read_if('5'), true);

		assert_eq!(cursor.span_from(&pos).text(), "45");

		assert_eq!(cursor.read(), Some('6'));
		assert_eq!(cursor.read(), None);
		assert_eq!(cursor.at_end(), true);

		Ok(())
	}
}
