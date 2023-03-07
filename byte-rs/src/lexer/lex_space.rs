use super::{Lexer, LexerResult, Reader};

pub struct LexSpace;

impl Lexer for LexSpace {
	fn read(&self, next: char, input: &mut Reader) -> LexerResult {
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
				LexerResult::Skip
			}

			_ => LexerResult::None,
		}
	}
}
