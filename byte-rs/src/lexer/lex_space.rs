use super::{Input, LexResult, LexValue, Lexer, Reader};

pub struct TokenSpace<T: LexValue>(pub T);

impl<T: LexValue> Lexer<T> for TokenSpace<T> {
	fn read<S: Input>(&self, next: char, input: &mut Reader<S>) -> LexResult<T> {
		match next {
			' ' | '\t' => {
				let mut pos;
				loop {
					pos = input.save();
					match input.read() {
						Some(' ' | '\t') => {}
						_ => break,
					}
				}
				input.restore(pos);
				LexResult::Ok(self.0)
			}

			_ => LexResult::None,
		}
	}
}
