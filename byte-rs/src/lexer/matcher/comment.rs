use super::{Cursor, Matcher, MatcherResult};

pub struct MatchComment;

impl Matcher for MatchComment {
	fn try_match(&self, next: char, input: &mut Cursor) -> MatcherResult {
		match next {
			'#' => {
				let (multi, mut level) = if input.read_if('(') {
					(true, 1)
				} else {
					(false, 0)
				};

				let mut pos;
				let putback = loop {
					pos = *input;
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
					*input = pos;
				}
				MatcherResult::Comment
			}

			_ => MatcherResult::None,
		}
	}
}
