use super::{Input, LexResult, LexValue, Lexer, Reader};

pub struct TokenLineBreak<T: LexValue>(pub T);

impl<T: LexValue> Lexer for TokenLineBreak<T> {
	type Value = T;

	fn read<S: Input>(&self, next: char, input: &mut Reader<S>) -> LexResult<T> {
		match next {
			'\r' => {
				input.read_if('\n');
				LexResult::Ok(self.0)
			}

			'\n' => LexResult::Ok(self.0),

			_ => LexResult::None,
		}
	}
}
