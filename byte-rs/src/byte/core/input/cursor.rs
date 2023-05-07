use super::*;

/// Cursor indexes a position in an [`Input`] and provides methods for reading
/// characters from that position forward.
///
/// This type is lightweight and can be easily copied to save and backtrack to
/// an input position.
#[derive(Clone, Default)]
pub struct Cursor {
	input: Input,
	row: usize,
	col: usize,
	offset: usize,
	indent: usize,
}

impl Cursor {
	pub fn from(input: &Input) -> Self {
		Cursor {
			input: input.clone(),
			row: 0,
			col: 0,
			offset: 0,
			indent: 0,
		}
	}

	/// True if the current position is at the start of the line.
	pub fn is_new_line(&self) -> bool {
		self.col == 0
	}

	/// Source input.
	pub fn input(&self) -> &Input {
		&self.input
	}

	/// Line number for the current position starting at 1.
	pub fn line(&self) -> usize {
		self.row + 1
	}

	/// Column number for the current position.
	///
	/// The column number starts at 1 and increments by 1 for each character,
	/// except for tabs which will set the column to the next tab stop column
	/// given by [`TAB_WIDTH`].
	pub fn column(&self) -> usize {
		self.col + 1
	}

	/// Byte offset for the current position from the start of the input.
	pub fn offset(&self) -> usize {
		self.offset
	}

	/// Indentation level for the current line.
	///
	/// This is zero at the start of the line and is incremented for each
	/// leading space character, until a non-space character is found.
	///
	/// As with `column()`, tab characters will expand to the next tab stop
	/// as given by [`TAB_WIDTH`].
	pub fn indent(&self) -> usize {
		self.indent
	}

	/// True when not at the end of the input.
	pub fn is_some(&self) -> bool {
		self.offset < self.input.len()
	}

	/// True at the end of the input.
	pub fn is_end(&self) -> bool {
		!self.is_some()
	}

	/// Read the next character in the input.
	pub fn read(&mut self) -> Option<char> {
		let text = self.input.text(self.offset..);
		if let Some(next) = text.chars().next() {
			// increment indentation until we find the first non-space character
			let is_leading_space = self.indent == self.col;

			// update offset to next char
			self.offset += next.len_utf8();

			// translate CR and CR+LF to a single LF
			let next = if next == '\r' && text.len() > 1 {
				if text.as_bytes()[1] == '\n' as u8 {
					self.offset += 1;
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
			if next == '\n' || (is_space(next) && is_leading_space) {
				self.indent = self.col;
			}
			Some(next)
		} else {
			None
		}
	}

	pub fn peek(&self) -> Option<char> {
		let mut copy = self.clone();
		copy.read()
	}

	pub fn try_read(&mut self, expected: char) -> bool {
		let mut copy = self.clone();
		if let Some(next) = copy.read() {
			if next == expected {
				*self = copy;
				return true;
			}
		}
		false
	}
}

//====================================================================================================================//
// Traits
//====================================================================================================================//

impl std::fmt::Display for Cursor {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}:{}", self.line(), self.column())
	}
}

impl std::fmt::Debug for Cursor {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "<pos {}:{self}>", self.input)
	}
}

impl PartialEq for Cursor {
	fn eq(&self, other: &Self) -> bool {
		self.input == other.input && self.offset == other.offset
	}
}

impl Eq for Cursor {}
