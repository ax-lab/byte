use std::path::Path;

/// Tab-width considered when computing column and indentation.
pub const TAB_WIDTH: usize = 4;

/// This is used for the lexer to determined what is a whitespace character.
pub fn is_space(char: char) -> bool {
	matches!(char, ' ' | '\t')
}

/// Open a file as input.
pub fn open_file<P: AsRef<Path>>(path: P) -> std::io::Result<Input> {
	let path = path.as_ref();
	let data = std::fs::read(path)?;
	let name = to_static(&path.to_string_lossy());
	let data = data.into_boxed_slice();
	let data = Box::leak(data);
	Ok(Input { name, data })
}

/// Open a plain string as input. The string is copied.
pub fn open_text<S: AsRef<str>>(name: &str, text: S) -> Input {
	let name = to_static(name);
	let text = to_static(text.as_ref());
	Input {
		name,
		data: text.as_bytes(),
	}
}

/// Open a static string as input. This does not copy the string.
pub fn open_str(name: &str, text: &'static str) -> Input {
	let name = to_static(name);
	Input {
		name,
		data: text.as_bytes(),
	}
}

/// Input source for the compiler.
#[derive(Copy, Clone)]
pub struct Input {
	name: &'static str,
	data: &'static [u8],
}

impl Input {
	pub fn name(&self) -> &'static str {
		self.name
	}

	pub fn sta(&self) -> Cursor {
		Cursor {
			src: *self,
			pos: Pos::LineCol(0, 0),
			offset: 0,
			indent: 0,
		}
	}

	pub fn end(&self) -> Cursor {
		Cursor {
			src: *self,
			pos: Pos::EndOfInput,
			offset: self.data.len(),
			indent: 0,
		}
	}

	pub fn bytes(&self, span: Span) -> &'static [u8] {
		&self.data[span.pos.offset..span.end.offset]
	}

	pub fn text(&self, span: Span) -> &'static str {
		unsafe { std::str::from_utf8_unchecked(self.bytes(span)) }
	}
}

impl PartialEq for Input {
	fn eq(&self, other: &Self) -> bool {
		std::ptr::eq(self.name, other.name) && std::ptr::eq(self.data, other.data)
	}
}

impl Eq for Input {}

/// Span indexes a range of text from an [`Input`].
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Span {
	pub pos: Cursor,
	pub end: Cursor,
}

impl Span {
	pub fn src(&self) -> Input {
		self.pos.src()
	}

	pub fn text(&self) -> &'static str {
		self.src().text(*self)
	}
}

impl std::fmt::Debug for Span {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"[{}..{} @{} {}..{}]",
			self.pos.pos(),
			self.end.pos(),
			self.src().name(),
			self.pos.offset(),
			self.end.offset(),
		)
	}
}

impl std::fmt::Display for Span {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.pos)
	}
}

/// Cursor indexes a position in an [`Input`] and provides methods for reading
/// characters from that position forward.
///
/// This type is lightweight and can be easily copied to save and backtrack to
/// an input position.
#[derive(Copy, Clone)]
pub struct Cursor {
	src: Input,
	pos: Pos,
	offset: usize,
	indent: usize,
}

impl Cursor {
	/// Source input.
	pub fn src(&self) -> Input {
		self.src
	}

	pub fn pos(&self) -> Pos {
		self.pos
	}

	/// Byte offset from the start of the input.
	pub fn offset(&self) -> usize {
		self.offset
	}

	/// Column for the current position.
	pub fn column(&self) -> usize {
		self.pos().indent()
	}

	/// Indentation level.
	///
	/// This is the width of the leading whitespace in the current line up to
	/// the current position.
	///
	/// For the purposes of indentation, TAB will expand the width to the next
	/// multiple of the [`TAB_WIDTH`] and other whitespace count as one.
	pub fn indent(&self) -> usize {
		self.indent
	}

	pub fn read(&mut self) -> Option<char> {
		let text = unsafe { std::str::from_utf8_unchecked(&self.src.data[self.offset..]) };
		if let Some(next) = text.chars().next() {
			let is_start = self.indent == self.pos.indent();
			self.offset += next.len_utf8();
			let next = if next == '\r' {
				let mut next = *self;
				if next.read() == Some('\n') {
					*self = next;
				}
				'\n'
			} else {
				next
			};
			self.pos.advance(next);
			if next == '\n' || is_space(next) && is_start {
				self.indent = self.pos.indent();
			}
			Some(next)
		} else {
			self.pos = Pos::EndOfInput;
			self.indent = 0;
			None
		}
	}

	pub fn peek(&self) -> Option<char> {
		let mut cursor = *self;
		cursor.read()
	}

	pub fn read_if(&mut self, expected: char) -> bool {
		let mut cursor = *self;
		if cursor.read() == Some(expected) {
			*self = cursor;
			true
		} else {
			false
		}
	}
}

impl PartialEq for Cursor {
	fn eq(&self, other: &Self) -> bool {
		self.src == other.src && self.offset == other.offset
	}
}

impl Eq for Cursor {}

impl std::fmt::Display for Cursor {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.pos)
	}
}

impl std::fmt::Debug for Cursor {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "[{} @{}:{}]", self.pos(), self.src.name(), self.offset)
	}
}

/// Represents a line/column position in a source input.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Pos {
	/// Zero-based line and column position, considering [`TAB_WIDTH`].
	LineCol(usize, usize),
	/// End of input.
	EndOfInput,
}

impl Pos {
	fn indent(&self) -> usize {
		match self {
			Pos::LineCol(_, col) => *col,
			Pos::EndOfInput => 0,
		}
	}

	fn advance(&mut self, next: char) {
		match self {
			Pos::LineCol(line, col) => {
				if next == '\n' {
					*line += 1;
					*col = 0;
				} else if next == '\t' {
					*col += 4 - (*col % 4)
				} else {
					*col += 1;
				}
			}
			Pos::EndOfInput => {}
		}
	}
}

impl std::fmt::Display for Pos {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Pos::LineCol(line, col) => write!(f, "L{}:{:02}", line + 1, col + 1),
			Pos::EndOfInput => write!(f, "end of input"),
		}
	}
}

/// Transform the given input string to a static string by moving it
/// into heap memory and never deallocating it.
///
/// For simplicity sake we use this for source text which is kept
/// alive during the entire compiler lifetime.
fn to_static(input: &str) -> &'static str {
	let input = input.to_string();
	let input = input.into_boxed_str();
	Box::leak(input)
}
