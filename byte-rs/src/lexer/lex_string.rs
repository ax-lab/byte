use super::{Lexer, LexerResult, Reader, Token};

pub struct LexString<F: Fn(String) -> Token>(pub F);

impl<F: Fn(String) -> Token> Lexer for LexString<F> {
	fn read(&self, next: char, input: &mut Reader) -> LexerResult {
		match next {
			'\'' => {
				let mut text = String::new();
				loop {
					match input.read() {
						Some('\'') => {
							break LexerResult::Token(self.0(text));
						}

						None => break LexerResult::Error("unclosed string literal".into()),

						Some(char) => {
							text.push(char);
						}
					}
				}
			}

			_ => LexerResult::None,
		}
	}
}
