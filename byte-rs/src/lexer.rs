#[derive(Eq, PartialEq, Debug)]
pub enum TokenKind {
	Identifier,
	Integer,
	String,
	Symbol,
	EndOfFile,
	Comma,
	LineBreak,
	Invalid,
	Error(String),
}

pub trait Input {
	fn read(&mut self) -> Option<char>;
	fn putback(&mut self);
	fn error(&self) -> Option<std::io::Error>;

	fn read_if(&mut self, next: char) -> bool {
		if self.read() == Some(next) {
			true
		} else {
			self.putback();
			false
		}
	}
}

pub fn read_token<I: Input>(input: &mut I) -> Option<TokenKind> {
	let token = match input.read() {
		Some(',') => TokenKind::Comma,

		Some(' ' | '\t') => {
			loop {
				match input.read() {
					Some(' ' | '\t') => {}
					_ => break,
				}
			}
			input.putback();
			return None;
		}

		Some('\r') => {
			input.read_if('\n');
			TokenKind::LineBreak
		}

		Some('\n') => TokenKind::LineBreak,

		Some('0'..='9') => {
			loop {
				match input.read() {
					Some('0'..='9') => {}
					_ => {
						break;
					}
				}
			}
			input.putback();
			TokenKind::Integer
		}

		Some('a'..='z' | 'A'..='Z' | '_') => {
			loop {
				match input.read() {
					Some('a'..='z' | 'A'..='Z' | '_' | '0'..='9') => {}
					_ => {
						break;
					}
				}
			}
			input.putback();
			TokenKind::Identifier
		}

		Some('+' | '-' | '*' | '/' | '=') => TokenKind::Symbol,

		Some('\'') => loop {
			match input.read() {
				Some('\'') => {
					break TokenKind::String;
				}

				None => {
					break TokenKind::Invalid;
				}

				_ => {}
			}
		},

		None => {
			if let Some(err) = input.error() {
				TokenKind::Error(err.to_string())
			} else {
				TokenKind::EndOfFile
			}
		}

		_ => TokenKind::Invalid,
	};

	Some(token)
}
