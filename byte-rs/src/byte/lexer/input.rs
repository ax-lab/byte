use std::path::Path;

use super::*;

//====================================================================================================================//
// Input
//====================================================================================================================//

/// Default tab width across the compiler.
///
/// This affects reported column numbers and the computed indentation values.
pub const DEFAULT_TAB_WIDTH: usize = 4;

/// Generic input source.
#[derive(Clone)]
pub struct Input {
	data: Arc<(String, Vec<u8>)>,
	tab_width: usize,
	line: usize,
	column: usize,
	indent: usize,
}

impl Input {
	pub fn new<T: Into<String>>(name: T, data: Vec<u8>) -> Self {
		let name = name.into();
		let data = Arc::new((name, data));
		Self {
			data,
			tab_width: DEFAULT_TAB_WIDTH,
			line: 0,
			column: 0,
			indent: 0,
		}
	}

	/// Open a file.
	pub fn open<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
		let path = path.as_ref();
		let data = std::fs::read(path)?;
		let name = path.to_string_lossy().to_string();
		Ok(Self::new(name, data))
	}

	pub fn with_pos(self, line: usize, column: usize, indent: usize) -> Self {
		let mut input = self;
		input.line = line - 1;
		input.column = column - 1;
		input.indent = indent;
		input
	}

	pub fn start(&self) -> Cursor {
		Cursor {
			source: self.clone(),
			line: self.line,
			column: self.column,
			indent: self.indent,
			offset: 0,
		}
	}

	pub fn len(&self) -> usize {
		self.data().len()
	}

	pub fn name(&self) -> &str {
		&self.data.0
	}

	pub fn data(&self) -> &[u8] {
		&self.data.1
	}

	pub fn text(&self) -> &str {
		std::str::from_utf8(self.data()).unwrap()
	}

	pub fn tab_width(&self) -> usize {
		self.tab_width
	}
}

impl Debug for Input {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		let name = self.name();
		let len = self.len();
		if len <= 80 {
			write!(f, "<Input {name}: {:?}>", self.text())
		} else {
			write!(f, "<Input {name} ({} bytes)>", len)
		}
	}
}

impl PartialEq for Input {
	fn eq(&self, other: &Self) -> bool {
		if Arc::as_ptr(&self.data) == Arc::as_ptr(&other.data) {
			self.line == other.line
				&& self.column == other.column
				&& self.indent == other.indent
				&& self.tab_width == other.tab_width
		} else {
			false
		}
	}
}

impl Eq for Input {}

//====================================================================================================================//
// Cursor
//====================================================================================================================//

/// Indexes a position in an [`Input`] and provides methods for reading it.
#[derive(Clone)]
pub struct Cursor {
	source: Input,
	line: usize,
	column: usize,
	indent: usize,
	offset: usize,
}

impl Cursor {
	pub fn source(&self) -> &Input {
		&self.source
	}

	pub fn with_pos(&self, line: usize, column: usize, indent: usize) -> Self {
		let mut cursor = self.clone();
		cursor.line = line - 1;
		cursor.column = column - 1;
		cursor.indent = indent;
		cursor
	}

	pub fn at_end(&self) -> bool {
		self.data().len() == 0
	}

	pub fn data(&self) -> &[u8] {
		let data = self.source().data();
		&data[self.offset..]
	}

	pub fn line(&self) -> usize {
		self.line + 1
	}

	pub fn is_indent(&self) -> bool {
		self.column == self.indent
	}

	pub fn column(&self) -> usize {
		self.column + 1
	}

	pub fn indent(&self) -> usize {
		self.indent
	}

	pub fn offset(&self) -> usize {
		self.offset
	}

	pub fn read(&mut self) -> Option<char> {
		if let Some((char, size)) = self.next_char() {
			self.advance(size);
			Some(char)
		} else {
			None
		}
	}

	pub fn try_read(&mut self, expected: char) -> bool {
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

	pub fn next_char(&self) -> Option<(char, usize)> {
		let data = self.source().data();
		let offset = self.offset;
		if offset < data.len() {
			let size = char_size(&data[offset..]);
			let char = &data[offset..offset + size];
			let char = std::str::from_utf8(char)
				.ok()
				.and_then(|x| x.chars().next())
				.unwrap_or('\u{FFFD}');
			Some((char, size))
		} else {
			None
		}
	}

	pub fn data_from<'a>(&self, start: &'a Cursor) -> &'a [u8] {
		assert!(self.offset() > start.offset());
		let offset = self.offset() - start.offset();
		let data = start.data();
		&data[..offset]
	}

	pub fn advance(&mut self, length: usize) {
		let tab_width = self.source().tab_width();
		let data = self.source().data();

		let mut skip = &data[self.offset..(self.offset + length)];
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
		self.offset += length;
	}
}
