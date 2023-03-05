use super::{Lexer, LexerResult, Reader, Token};

pub struct LexNumber<F: Fn(u64) -> Token>(pub F);

impl<F: Fn(u64) -> Token> Lexer for LexNumber<F> {
	fn read(&self, next: char, input: &mut Reader) -> LexerResult {
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
