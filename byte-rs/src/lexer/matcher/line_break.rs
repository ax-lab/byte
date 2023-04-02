use crate::core::input::*;

use super::{Matcher, MatcherResult, Token};

#[derive(Clone)]
pub struct MatchLineBreak(pub Token);

impl Matcher for MatchLineBreak {
	fn try_match(&self, next: char, _input: &mut Cursor) -> MatcherResult {
		match next {
			'\n' => MatcherResult::Token(self.0.clone()),

			_ => MatcherResult::None,
		}
	}

	fn clone_box(&self) -> Box<dyn Matcher> {
		Box::new(self.clone())
	}
}
