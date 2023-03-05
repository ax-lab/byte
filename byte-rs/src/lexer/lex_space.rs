use super::{IsToken, Lexer, LexerResult, Reader};

pub struct LexSpace<T: IsToken>(pub T);

impl<T: IsToken> Lexer<T> for LexSpace<T> {
	fn read(&self, next: char, input: &mut Reader) -> LexerResult<T> {
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
				LexerResult::Token(self.0.clone())
			}

			_ => LexerResult::None,
		}
	}
}
