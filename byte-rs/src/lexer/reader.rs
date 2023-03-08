use std::rc::Rc;

use super::Input;
use super::Pos;

/// Wrapper for an [Input] providing support for lexing.
#[derive(Clone)]
pub struct Reader {
	inner: Rc<Box<dyn Input + 'static>>,
	pos: Pos,
	was_cr: bool,
}

impl<'a, T: Input + 'static> From<T> for Reader {
	fn from(value: T) -> Self {
		Reader {
			inner: Rc::new(Box::new(value)),
			pos: Default::default(),
			was_cr: false,
		}
	}
}

impl<'a> Reader {
	pub fn read_text(&self, pos: usize, end: usize) -> &str {
		let inner = &self.inner;
		inner.read_text(pos, end)
	}

	/// Return the current position for the reader.
	pub fn pos(&self) -> Pos {
		self.pos
	}

	/// Read the next character in the input and advances its position.
	pub fn read(&mut self) -> Option<char> {
		let len = self.inner.len();
		let txt = self.inner.read_text(self.pos.offset, len);
		let mut chars = txt.char_indices();
		if let Some((_, next)) = chars.next() {
			let offset = chars.next().map(|x| self.pos.offset + x.0).unwrap_or(len);
			self.advance(next, offset);
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
	}

	/// Read the next character in the input if it is the given character.
	pub fn read_if(&mut self, expected: char) -> bool {
		let pos = self.save();
		if let Some(next) = self.read() {
			if next == expected {
				return true;
			}
		}
		self.restore(pos);
		false
	}

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
