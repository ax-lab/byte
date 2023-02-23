use super::{Input, LexResult, LexValue, Lexer, Reader};

pub struct TokenIdentifier<T: LexValue>(pub T);

impl<T: LexValue> Lexer for TokenIdentifier<T> {
	type Value = T;

	fn read<S: Input>(&self, next: char, input: &mut Reader<S>) -> LexResult<T> {
		match next {
			'a'..='z' | 'A'..='Z' | '_' => {
				let mut pos;
				loop {
					pos = input.save();
					match input.read() {
						Some('a'..='z' | 'A'..='Z' | '_' | '0'..='9') => {}
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
