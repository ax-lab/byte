use super::Pos;

/// Trait implemented by any input to a [super::Lexer].
pub trait Input {
	type Error: std::fmt::Display;

	fn offset(&self) -> usize;
	fn set_offset(&mut self, pos: usize);

	fn read(&mut self) -> Option<char>;
	fn read_text(&self, pos: usize, end: usize) -> &str;
}

/// Wrapper for an [Input] providing support for lexing.
pub struct Reader<T: Input> {
	inner: T,
	pos: Pos,
	was_cr: bool,
}

impl<T: Input> From<T> for Reader<T> {
	fn from(value: T) -> Self {
		Reader {
			inner: value,
			pos: Default::default(),
			was_cr: false,
		}
	}
}

impl<T: Input> Reader<T> {
	pub fn inner(&self) -> &T {
		&self.inner
	}

	/// Return the current position for the reader.
	pub fn pos(&self) -> Pos {
		self.pos
	}

	/// Read the next character in the input and advances its position.
	pub fn read(&mut self) -> Option<char> {
		if let Some(next) = self.inner.read() {
			self.advance(next, self.inner.offset());
			Some(next)
		} else {
			None
		}
	}

	/// Returns the current state of the reader for backtracking.
	pub fn save(&self) -> (Pos, bool) {
		(self.pos, self.was_cr)
	}

	/// Backtrack to a state saved by [`save()`].
	pub fn restore(&mut self, state: (Pos, bool)) {
		(self.pos, self.was_cr) = state;
		self.inner.set_offset(self.pos.offset);
	}

	/// Read the next character in the input if it is the given character.
	pub fn read_if(&mut self, expected: char) -> bool {
		let offset = self.inner.offset();
		if let Some(next) = self.inner.read() {
			if next == expected {
				self.advance(next, self.inner.offset());
				return true;
			}
		}
		self.inner.set_offset(offset);
		false
	}

	/// Advance the reader position based on the given character.
	fn advance(&mut self, next: char, offset: usize) {
		match next {
			'\n' => {
				if !self.was_cr {
					self.pos.line += 1;
					self.pos.column = 0;
				}
			}
			'\t' => {
				self.pos.column += 4 - (self.pos.column % 4);
			}
			_ => {
				self.pos.column += 1;
			}
		}
		self.pos.offset = offset;
		self.was_cr = next == '\r';
	}
}
