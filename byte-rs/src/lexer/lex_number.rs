use super::{Input, IsToken, Lexer, LexerResult, Reader};

pub struct LexNumber<T: IsToken>(pub T);

impl<T: IsToken> Lexer<T> for LexNumber<T> {
	fn read<S: Input>(&self, next: char, input: &mut Reader<S>) -> LexerResult<T> {
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
				LexerResult::Token(self.0.clone())
			}

			_ => LexerResult::None,
		}
	}
}
