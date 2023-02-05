pub enum Token {
	Identifier,
	Integer,
	Symbol,
	EndOfFile,
	Comma,
	LineBreak,
	Invalid,
	Error(std::io::Error),
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

pub fn read_token<I: Input>(input: &mut I) -> Token {
	match input.read() {
		Some(',') => Token::Comma,

		Some('\r') => {
			input.read_if('\n');
			Token::LineBreak
		}

		Some('\n') => Token::LineBreak,

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
			Token::Integer
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
			Token::Identifier
		}

		Some('+' | '-' | '*' | '/' | '=') => Token::Symbol,

		None => {
			if let Some(err) = input.error() {
				Token::Error(err)
			} else {
				Token::EndOfFile
			}
		}

		_ => Token::Invalid,
	}
}
