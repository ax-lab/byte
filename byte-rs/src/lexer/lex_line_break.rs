use super::{Cursor, Matcher, MatcherResult, Token};

pub struct LexLineBreak(pub Token);

impl Matcher for LexLineBreak {
	fn try_match(&self, next: char, input: &mut Cursor) -> MatcherResult {
		match next {
			'\r' => {
				input.read_if('\n');
				MatcherResult::Token(self.0.clone())
			}

			'\n' => MatcherResult::Token(self.0.clone()),

			_ => MatcherResult::None,
		}
	}
}
