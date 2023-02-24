use super::{Input, IsToken, Lexer, LexerResult, Reader};

pub struct LexString<T: IsToken>(pub T);

impl<T: IsToken> Lexer<T> for LexString<T> {
	fn read<S: Input>(&self, next: char, input: &mut Reader<S>) -> LexerResult<T> {
		match next {
			'\'' => loop {
				match input.read() {
					Some('\'') => {
						break LexerResult::Token(self.0.clone());
					}

					None => break LexerResult::Error("unclosed string literal".into()),

					_ => {}
				}
			},

			_ => LexerResult::None,
		}
	}
}
