use super::token::Token;

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

pub fn read_token<I: Input>(input: &mut I) -> (Token, bool) {
	match input.read() {
		Some(',') => (Token::Comma, true),

		Some(' ' | '\t') => {
			loop {
				match input.read() {
					Some(' ' | '\t') => {}
					_ => break,
				}
			}
			input.putback();
			return (Token::None, true);
		}

		Some('#') => {
			let (multi, mut level) = if input.read_if('(') {
				(true, 1)
			} else {
				(false, 0)
			};

			let putback = loop {
				match input.read() {
					Some('\n' | '\r') if !multi => break true,
					Some('(') if multi => {
						level += 1;
					}
					Some(')') if multi => {
						level -= 1;
						if level == 0 {
							break false;
						}
					}
					Some(_) => {}
					None => break false,
				}
			};
			if putback {
				input.putback();
			}
			return (Token::Comment, true);
		}

		Some('\r') => {
			input.read_if('\n');
			(Token::LineBreak, true)
		}

		Some('\n') => (Token::LineBreak, true),

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
			(Token::Integer, true)
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
			(Token::Identifier, true)
		}

		Some('.') => {
			input.read_if('.');
			(Token::Symbol, true)
		}

		Some('=') => {
			input.read_if('=');
			(Token::Symbol, true)
		}

		Some('+' | '-' | '*' | '/' | '%' | '?' | ':' | '(' | ')') => (Token::Symbol, true),

		Some('\'') => loop {
			match input.read() {
				Some('\'') => {
					break (Token::String, true);
				}

				None => {
					break (Token::Symbol, true);
				}

				_ => {}
			}
		},

		None => {
			if let Some(err) = input.error() {
				panic!("input error: {err}");
			} else {
				(Token::None, false)
			}
		}

		Some(char) => panic!("invalid symbol: {char:?}"),
	}
}
