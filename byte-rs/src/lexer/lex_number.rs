use super::{Input, LexResult, LexValue, Lexer, Reader};

pub struct TokenNumber<T: LexValue>(pub T);

impl<T: LexValue> Lexer<T> for TokenNumber<T> {
	fn read<S: Input>(&self, next: char, input: &mut Reader<S>) -> LexResult<T> {
		match next {
			'0'..='9' => {
				let mut pos;
				loop {
					pos = input.save();
					match input.read() {
						Some('0'..='9') => {}
						_ => {
							break;
						}
					}
				}
				input.restore(pos);
				LexResult::Ok(self.0)
			}

			_ => LexResult::None,
		}
	}
}
