use super::{Input, IsToken, Lexer, LexerResult, Reader};

pub struct LexIdentifier<T: IsToken, F: Fn(String) -> T>(pub F);

impl<T: IsToken, F: Fn(String) -> T> Lexer<T> for LexIdentifier<T, F> {
	fn read<S: Input>(&self, next: char, input: &mut Reader<S>) -> LexerResult<T> {
		match next {
			'a'..='z' | 'A'..='Z' | '_' => {
				let mut pos;
				let mut id = String::new();
				id.push(next);
				loop {
					pos = input.save();
					match input.read() {
						Some(char @ ('a'..='z' | 'A'..='Z' | '_' | '0'..='9')) => {
							id.push(char);
						}
						_ => {
							input.restore(pos);
							break;
						}
					}
				}

				LexerResult::Token(self.0(id))
			}

			_ => LexerResult::None,
		}
	}
}
