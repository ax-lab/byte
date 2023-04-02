use std::{path::Path, rc::Rc};

use super::context::*;

/// Tab-width considered when computing column and indentation.
pub const TAB_WIDTH: usize = 4;

/// This is used for the lexer to determined what is a whitespace character.
pub fn is_space(char: char) -> bool {
	matches!(char, ' ' | '\t')
}

impl Context {
	/// Open a file as input.
	pub fn open_file<P: AsRef<Path>>(&self, path: P) -> std::io::Result<Input> {
		let path = path.as_ref();
		let data = std::fs::read(path)?;
		let name = path.to_string_lossy().to_string();
		let data = InputData { name, data };

		Ok(Input {
			internal: data.into(),
		})
	}

	/// Open a plain string as input. The string is copied.
	pub fn open_str<S: AsRef<str>>(&self, name: &str, text: S) -> Input {
		let text = self.save(text.as_ref().to_string());
		let data = InputData {
			name: name.to_string(),
			data: text.as_bytes().into(),
		};
		Input {
			internal: data.into(),
		}
	}
}

/// Input source for the compiler.
#[derive(Clone)]
pub struct Input {
	internal: Rc<InputData>,
}

struct InputData {
	name: String,
	data: Vec<u8>,
}

impl Input {
	pub fn name(&self) -> &str {
		&self.internal.name
	}

	pub fn start(&self) -> Cursor {
		Cursor {
			src: self.clone(),
			row: 0,
			col: 0,
			offset: 0,
			indent: 0,
		}
	}

	pub fn bytes(&self, span: &Span) -> &[u8] {
		&self.internal.data[span.sta.offset..span.end.offset]
	}

	pub fn text(&self, span: &Span) -> &str {
		unsafe { std::str::from_utf8_unchecked(self.bytes(span)) }
	}
}

impl PartialEq for Input {
	fn eq(&self, other: &Self) -> bool {
		let a = &*self.internal;
		let b = &*other.internal;
		std::ptr::eq(a, b)
	}
}

impl Eq for Input {}

impl std::fmt::Display for Input {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.internal.name)
	}
}

/// Span indexes a range of text from an [`Input`].
#[derive(Clone, Eq, PartialEq)]
pub struct Span {
	pub sta: Cursor,
	pub end: Cursor,
}

impl Span {
	pub fn src(&self) -> &Input {
		self.sta.src()
	}

	pub fn text(&self) -> &str {
		self.sta.src.text(self)
	}
}

impl std::fmt::Debug for Span {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"[{}~{} @{} {}..{}]",
			self.sta,
			self.end,
			self.src().name(),
			self.sta.offset(),
			self.end.offset(),
		)
	}
}

impl std::fmt::Display for Span {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.sta)
	}
}

/// Cursor indexes a position in an [`Input`] and provides methods for reading
/// characters from that position forward.
///
/// This type is lightweight and can be easily copied to save and backtrack to
/// an input position.
#[derive(Clone)]
pub struct Cursor {
	src: Input,
	row: usize,
	col: usize,
	offset: usize,
	indent: usize,
}

impl Cursor {
	/// Source input.
	pub fn src(&self) -> &Input {
		&self.src
	}

	pub fn row(&self) -> usize {
		self.row
	}

	pub fn col(&self) -> usize {
		self.col
	}

	/// Byte offset from the start of the input.
	pub fn offset(&self) -> usize {
		self.offset
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
		let text = unsafe { std::str::from_utf8_unchecked(&self.src.internal.data[self.offset..]) };
		if let Some(next) = text.chars().next() {
			// keep indentation until we find the first non-space character
			let is_line_indent = self.indent == self.col;

			// update offset to next char
			self.offset += next.len_utf8();

			// translate CR and CR+LF to a single LF
			let next = if next == '\r' {
				let mut next = self.clone();
				if next.read() == Some('\n') {
					*self = next;
				}
				'\n'
			} else {
				next
			};

			// update position
			if next == '\n' {
				self.row += 1;
				self.col = 0;
			} else if next == '\t' {
				self.col += TAB_WIDTH - (self.col % TAB_WIDTH)
			} else {
				self.col += 1;
			}

			// update indentation
			if next == '\n' || (is_space(next) && is_line_indent) {
				self.indent = self.col;
			}
			Some(next)
		} else {
			None
		}
	}

	//------------------------------------------------------------------------//
	// Utility functions
	//------------------------------------------------------------------------//

	pub fn read_if(&mut self, expected: char) -> bool {
		let mut cursor = self.clone();
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
		write!(f, "L{}:{:02}", self.row() + 1, self.col() + 1)
	}
}

impl std::fmt::Debug for Cursor {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self)
	}
}
