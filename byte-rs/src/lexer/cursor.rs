use super::Input;

/// Indexes a position in an [Input] and provides methods for consuming it.
///
/// This type is designed to have lightweight copy semantics which allows for
/// easily saving a position and backtracking to it. It also holds a reference
/// to the source, allowing easy access to the source text.
#[derive(Clone, Copy)]
pub struct Cursor<'a> {
	pub source: &'a dyn Input,
	pub line: usize,
	pub column: usize,
	pub offset: usize,

	was_cr: bool,
}

impl<'a> Cursor<'a> {
	pub fn new(source: &'a dyn Input) -> Self {
		Cursor {
			source,
			line: 0,
			column: 0,
			offset: 0,
			was_cr: false,
		}
	}

	/// Read the next character in the input and advances its position.
	pub fn read(&mut self) -> Option<char> {
		let len = self.source.len();
		let txt = self.source.read_text(self.offset, len);
		let mut chars = txt.char_indices();
		if let Some((_, next)) = chars.next() {
			let offset = chars.next().map(|x| self.offset + x.0).unwrap_or(len);
			self.advance(next, offset);
			Some(next)
		} else {
			None
		}
	}

	/// Read the next character in the input if it is the given character.
	pub fn read_if(&mut self, expected: char) -> bool {
		let pos = *self;
		if let Some(next) = self.read() {
			if next == expected {
				return true;
			}
		}
		*self = pos;
		false
	}

	fn advance(&mut self, next: char, offset: usize) {
		match next {
			'\n' => {
				if !self.was_cr {
					self.line += 1;
					self.column = 0;
				}
			}
			'\t' => {
				self.column += 4 - (self.column % 4);
			}
			_ => {
				self.column += 1;
			}
		}
		self.offset = offset;
		self.was_cr = next == '\r';
	}
}

impl<'a> PartialEq for Cursor<'a> {
	fn eq(&self, other: &Self) -> bool {
		let same_source = std::ptr::eq(self.source, other.source);
		same_source
			&& self.line == other.line
			&& self.column == other.column
			&& self.offset == other.offset
			&& self.was_cr == other.was_cr
	}
}

impl<'a> Eq for Cursor<'a> {}

impl<'a> std::fmt::Display for Cursor<'a> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let line = self.line + 1;
		let column = self.column + 1;
		write!(f, "{line:03},{column:02}")
	}
}

impl<'a> std::fmt::Debug for Cursor<'a> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Cursor")
			.field("source", &format_args!("{:?}", &self.source as *const _))
			.field("line", &self.line)
			.field("column", &self.column)
			.field("offset", &self.offset)
			.field("was_cr", &self.was_cr)
			.finish()
	}
}
