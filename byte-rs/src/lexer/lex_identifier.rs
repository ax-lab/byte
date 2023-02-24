use super::{Input, IsToken, Lexer, LexerResult, Reader};

pub struct LexIdentifier<T: IsToken>(pub T);

impl<T: IsToken> Lexer<T> for LexIdentifier<T> {
	fn read<S: Input>(&self, next: char, input: &mut Reader<S>) -> LexerResult<T> {
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
				LexerResult::Token(self.0.clone())
			}

			_ => LexerResult::None,
		}
	}
}
