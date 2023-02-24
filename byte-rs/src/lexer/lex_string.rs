use super::{Input, IsToken, Lexer, LexerResult, Reader};

pub struct LexString<T: IsToken, F: Fn(String) -> T>(pub F);

impl<T: IsToken, F: Fn(String) -> T> Lexer<T> for LexString<T, F> {
	fn read<S: Input>(&self, next: char, input: &mut Reader<S>) -> LexerResult<T> {
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
