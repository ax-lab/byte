use super::{Lexer, LexerResult, Reader, Token};

pub struct LexIdentifier<F: Fn(String) -> Token>(pub F);

impl<F: Fn(String) -> Token> Lexer for LexIdentifier<F> {
	fn read(&self, next: char, input: &mut Reader) -> LexerResult {
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
