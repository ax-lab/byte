use super::{Input, LexResult, Lexer, Reader};

pub struct TokenSpace;

impl Lexer for TokenSpace {
	type Value = ();

	fn read<S: Input>(&self, next: char, input: &mut Reader<S>) -> LexResult<()> {
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
				LexResult::Ok(())
			}

			_ => LexResult::None,
		}
	}
}
