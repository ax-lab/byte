use super::{Input, IsToken, Lexer, LexerResult, Reader};

pub struct LexNumber<T: IsToken, F: Fn(u64) -> T>(pub F);

impl<T: IsToken, F: Fn(u64) -> T> Lexer<T> for LexNumber<T, F> {
	fn read<S: Input>(&self, next: char, input: &mut Reader<S>) -> LexerResult<T> {
		match next {
			'0'..='9' => {
				let mut value = decimal_value(next);
				let mut pos;
				loop {
					pos = input.save();
					match input.read() {
						Some(next @ '0'..='9') => {
							value = value * 10 + decimal_value(next);
						}
						_ => {
							break;
						}
					}
				}
				input.restore(pos);
				LexerResult::Token(self.0(value))
			}

			_ => LexerResult::None,
		}
	}
}

fn decimal_value(n: char) -> u64 {
	(n as u64) - ('0' as u64)
}
