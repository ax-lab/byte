use super::{Input, LexResult, LexValue, Lexer, Reader};

pub struct TokenString<T: LexValue>(pub T);

impl<T: LexValue> Lexer for TokenString<T> {
	type Value = T;

	fn read<S: Input>(&self, next: char, input: &mut Reader<S>) -> LexResult<T> {
		match next {
			'\'' => loop {
				match input.read() {
					Some('\'') => {
						break LexResult::Ok(self.0);
					}

					None => break LexResult::Error("unclosed string literal".into()),

					_ => {}
				}
			},

			_ => LexResult::None,
		}
	}
}
