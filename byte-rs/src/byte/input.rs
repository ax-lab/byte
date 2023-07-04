use super::*;

//====================================================================================================================//
// Span
//====================================================================================================================//

impl Source {
	pub fn span(&self) -> Span {
		Span::Some {
			source: self.clone(),
			offset: self.offset(),
			length: self.len(),
		}
	}

	pub fn start(&self) -> Cursor {
		self.span().start()
	}
}

/// Represents a range of source text from a [`SourceList`].
#[derive(Clone, Eq, PartialEq)]
pub enum Span {
	None,
	Some {
		source: Source,
		offset: usize,
		length: usize,
	},
}

impl Span {
	/// Create a new cursor at the start of the span.
	pub fn start(&self) -> Cursor {
		Cursor {
			span: self.clone(),
			source: self.source(),
			line: 0,
			column: 0,
			indent: 0,
		}
	}

	pub fn tab_width(&self) -> usize {
		match self {
			Span::None => DEFAULT_TAB_WIDTH,
			Span::Some { source, .. } => source.tab_width(),
		}
	}

	pub fn source(&self) -> Option<Source> {
		match self {
			Span::None => None,
			Span::Some { source, .. } => Some(source.clone()),
		}
	}

	/// Length of the span in bytes.
	pub fn len(&self) -> usize {
		match self {
			Span::None => 0,
			Span::Some { length, .. } => *length,
		}
	}

	/// Globally unique offset for the start position of this span across all
	/// source code.
	pub fn offset(&self) -> usize {
		match self {
			Span::None => 0,
			Span::Some { offset, .. } => *offset,
		}
	}

	/// Raw data for this span.
	pub fn data(&self) -> &[u8] {
		self.text().as_bytes()
	}

	/// Span for the full source text, if this is a partial span. Otherwise
	/// returns the current span itself.
	pub fn source_span(&self) -> Span {
		match self {
			Span::None => Span::None,
			Span::Some { source, .. } => Span::Some {
				source: source.clone(),
				offset: source.offset(),
				length: source.len(),
			},
		}
	}

	/// Text for this span.
	pub fn text(&self) -> &str {
		match self {
			Span::None => "",
			Span::Some { source, offset, length } => {
				let offset = offset - source.offset();
				&source.text()[offset..offset + length]
			}
		}
	}

	/// Name for the source of this span.
	pub fn source_name(&self) -> &str {
		match self {
			Span::None => "",
			Span::Some { source, .. } => source.name(),
		}
	}

	pub fn location(&self, tab_width: usize) -> Option<String> {
		let name = self.source_name();
		if name.len() > 0 {
			let location = if let Some((line, col)) = self.line_column(tab_width) {
				format!("{name}:{line}:{col}")
			} else {
				format!("{name}")
			};
			Some(location)
		} else {
			None
		}
	}

	/// Returns the line and column number for this span start location.
	pub fn line_column(&self, tab_width: usize) -> Option<(usize, usize)> {
		if let Some(source) = self.source() {
			let offset = self.offset() - source.offset();
			let prefix = &source.text().as_bytes()[..offset];
			let prefix = unsafe { std::str::from_utf8_unchecked(prefix) };

			let tab_width = if tab_width == 0 { DEFAULT_TAB_WIDTH } else { tab_width };

			let mut row = 0;
			let mut col = 0;
			let mut cr = false;
			for char in prefix.chars() {
				cr = if char == '\t' {
					col += tab_width - (col % tab_width);
					false
				} else if char == '\n' {
					col = 0;
					if !cr {
						row += 1;
					}
					false
				} else if char == '\r' {
					col = 0;
					row += 1;
					true
				} else {
					col += 1;
					false
				}
			}

			Some((row + 1, col + 1))
		} else {
			None
		}
	}
}

impl Debug for Span {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		const MAX_LEN: usize = 30;
		let name = self.source_name();
		let text = self.text();
		let chars = text.chars().count();

		if let Some(source) = self.source() {
			let offset = self.offset();
			let length = self.len();
			let sta = offset - source.offset();
			let end = sta + length;
			let len = length;

			let full = sta == 0 && length == source.len();

			write!(f, "<Span ")?;
			if chars <= MAX_LEN {
				write!(f, "{text:?}")?;
			} else {
				let text: String = text.chars().take(MAX_LEN).chain(std::iter::once('…')).collect();
				write!(f, "{text:?} ({len} bytes)")?;
			}
			write!(f, " @{name}")?;
			if !full {
				write!(f, "[{sta}…{end}]")?
			}
			write!(f, ">")
		} else {
			write!(f, "Span::None")
		}
	}
}

impl Hash for Span {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		match self {
			Span::None => 0.hash(state),
			Span::Some { source, offset, length } => {
				source.offset().hash(state);
				offset.hash(state);
				length.hash(state);
			}
		}
	}
}

//====================================================================================================================//
// Cursor
//====================================================================================================================//

/// Indexes a position in a [`Span`] and provides methods for reading its text.
#[derive(Clone)]
pub struct Cursor {
	#[allow(unused)]
	source: Option<Source>, // ensure source references are valid for the lifetime of the cursor
	span: Span,
	line: usize,
	column: usize,
	indent: usize,
}

impl Cursor {
	/// Span corresponding to the remaining input from the cursor position.
	pub fn span(&self) -> &Span {
		&self.span
	}

	/// Return the cursor current position as a span with length zero.
	pub fn pos_as_span(&self) -> Span {
		match self.span {
			Span::None => Span::None,
			Span::Some { ref source, offset, .. } => Span::Some {
				source: source.clone(),
				offset,
				length: 0,
			},
		}
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

	/// True if at the end of the input.
	pub fn at_end(&self) -> bool {
		self.data().len() == 0
	}

	/// Remaining input data from the cursor position.
	pub fn data(&self) -> &[u8] {
		self.span.data()
	}

	/// Remaining input text from the cursor position.
	pub fn text(&self) -> &str {
		self.span.text()
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
		let tab_width = self.span().tab_width();
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

		let skip_length = length;
		if let Span::Some {
			ref mut offset,
			ref mut length,
			..
		} = self.span
		{
			*offset += skip_length;
			*length -= skip_length;
		}
	}

	pub fn advance_span(&mut self, length: usize) -> Span {
		let start = self.clone();
		self.advance(length);
		self.span_from(&start)
	}

	/// Return a new [`Span`] from the given [`Cursor`] to the current position.
	///
	/// Both cursors MUST be from the same input source.
	pub fn span_from(&self, start: &Cursor) -> Span {
		match start.span {
			Span::None => {
				assert!(matches!(self.span, Span::None));
				Span::None
			}
			Span::Some {
				ref source,
				offset,
				length,
			} => {
				let my_offset = self.offset();
				assert!(my_offset >= offset && my_offset <= offset + length);
				Span::Some {
					source: source.clone(),
					offset: offset,
					length: my_offset - offset,
				}
			}
		}
	}
}

//====================================================================================================================//
// Tests
//====================================================================================================================//

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn cursors() -> Result<()> {
		let context = Context::get();
		let input = context.load_source_text("input A", "123456");

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
