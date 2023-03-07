use super::{Lexer, LexerResult, Reader};

pub struct LexComment;

impl Lexer for LexComment {
	fn read(&self, next: char, input: &mut Reader) -> LexerResult {
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
				LexerResult::Skip
			}

			_ => LexerResult::None,
		}
	}
}
