use super::{Input, IsToken, Lexer, LexerResult, Reader};

pub struct LexComment<T: IsToken>(pub T);

impl<T: IsToken> Lexer<T> for LexComment<T> {
	fn read<S: Input>(&self, next: char, input: &mut Reader<S>) -> LexerResult<T> {
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
				LexerResult::Token(self.0.clone())
			}

			_ => LexerResult::None,
		}
	}
}
