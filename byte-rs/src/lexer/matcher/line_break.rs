use super::{Cursor, Matcher, MatcherResult, Token};

#[derive(Copy, Clone)]
pub struct MatchLineBreak(pub Token);

impl Matcher for MatchLineBreak {
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

	fn clone_box(&self) -> Box<dyn Matcher> {
		Box::new(self.clone())
	}
}
