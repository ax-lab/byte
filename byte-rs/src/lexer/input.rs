use super::{Pos, Span};

pub trait Input {
	fn pos(&self) -> Pos;

	fn read(&mut self) -> Option<char>;
	fn rewind(&mut self, pos: Pos);
	fn error(&self) -> Option<std::io::Error>;

	fn read_text(&mut self, span: Span) -> &str;

	fn read_if(&mut self, next: char) -> bool {
		let pos = self.pos();
		if self.read() == Some(next) {
			true
		} else {
			self.rewind(pos);
			false
		}
	}
}
