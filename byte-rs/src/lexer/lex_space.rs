use super::{Cursor, Lexer, LexerResult};

pub struct LexSpace;

impl Lexer for LexSpace {
	fn read(&self, next: char, input: &mut Cursor) -> LexerResult {
		match next {
			' ' | '\t' => {
				let mut pos;
				loop {
					pos = *input;
					match input.read() {
						Some(' ' | '\t') => {}
						_ => break,
					}
				}
				*input = pos;
				LexerResult::Skip
			}

			_ => LexerResult::None,
		}
	}
}
