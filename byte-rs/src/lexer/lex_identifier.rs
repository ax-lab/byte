use super::{Lexer, LexerResult, Reader, Token};

pub struct LexIdentifier(pub Token);

impl Lexer for LexIdentifier {
	fn read(&self, next: char, input: &mut Reader) -> LexerResult {
		match next {
			'a'..='z' | 'A'..='Z' | '_' => {
				let mut pos;
				loop {
					pos = input.save();
					match input.read() {
						Some('a'..='z' | 'A'..='Z' | '_' | '0'..='9') => {}
						_ => {
							input.restore(pos);
							break;
						}
					}
				}

				LexerResult::Token(self.0)
			}

			_ => LexerResult::None,
		}
	}
}
