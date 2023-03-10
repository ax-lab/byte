use super::{Cursor, Lexer, LexerResult, Token};

pub struct LexIdentifier(pub Token);

impl Lexer for LexIdentifier {
	fn read(&self, next: char, input: &mut Cursor) -> LexerResult {
		match next {
			'a'..='z' | 'A'..='Z' | '_' => {
				let mut pos;
				loop {
					pos = *input;
					match input.read() {
						Some('a'..='z' | 'A'..='Z' | '_' | '0'..='9') => {}
						_ => {
							*input = pos;
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
