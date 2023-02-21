use super::{Pos, Span};

pub trait Input {
	type Error: std::fmt::Display;

	fn offset(&self) -> usize;
	fn set_offset(&mut self, pos: usize);

	fn read(&mut self) -> Option<char>;
	fn read_text(&mut self, from: usize, end: usize) -> &str;

	fn error(&self) -> Option<Self::Error>;
}

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

	#[allow(dead_code)]
	pub fn inner_mut(&mut self) -> &mut T {
		&mut self.inner
	}

	pub fn pos(&self) -> Pos {
		self.pos
	}

	pub fn read(&mut self) -> Option<char> {
		if let Some(next) = self.inner.read() {
			self.advance(next, self.inner.offset());
			Some(next)
		} else {
			None
		}
	}

	pub fn read_text(&mut self, span: Span) -> &str {
		self.inner.read_text(span.pos.offset, span.end.offset)
	}

	pub fn save(&self) -> (Pos, bool) {
		(self.pos, self.was_cr)
	}

	pub fn restore(&mut self, state: (Pos, bool)) {
		(self.pos, self.was_cr) = state;
		self.inner.set_offset(self.pos.offset);
	}

	pub fn read_if(&mut self, expected: char) -> bool {
		let pos = self.inner.offset();
		if let Some(next) = self.inner.read() {
			if next == expected {
				self.advance(next, self.inner.offset());
				return true;
			}
		}
		self.inner.set_offset(pos);
		false
	}

	pub fn error(&self) -> Option<String> {
		self.inner.error().map(|x| format!("{x}"))
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
