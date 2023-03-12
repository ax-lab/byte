use super::{Cursor, Matcher, MatcherResult, Token};

pub struct LexIdentifier(pub Token);

impl Matcher for LexIdentifier {
	fn try_match(&self, next: char, input: &mut Cursor) -> MatcherResult {
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

				MatcherResult::Token(self.0)
			}

			_ => MatcherResult::None,
		}
	}
}
