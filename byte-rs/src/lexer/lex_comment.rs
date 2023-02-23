use super::{Input, LexResult, LexValue, Lexer, Reader};

pub struct TokenComment<T: LexValue>(pub T);

impl<T: LexValue> Lexer<T> for TokenComment<T> {
	fn read<S: Input>(&self, next: char, input: &mut Reader<S>) -> LexResult<T> {
		match next {
			'#' => {
				let (multi, mut level) = if input.read_if('(') {
					(true, 1)
				} else {
					(false, 0)
				};

				let mut pos;
				let putback = loop {
					pos = input.save();
					match input.read() {
						Some('\n' | '\r') if !multi => break true,
						Some('(') if multi => {
							level += 1;
						}
						Some(')') if multi => {
							level -= 1;
							if level == 0 {
								break false;
							}
						}
						Some(_) => {}
						None => break false,
					}
				};
				if putback {
					input.restore(pos);
				}
				LexResult::Ok(self.0)
			}

			_ => LexResult::None,
		}
	}
}
