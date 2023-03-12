use super::{is_space, Cursor, Matcher, MatcherResult};

pub struct LexSpace;

impl Matcher for LexSpace {
	fn try_match(&self, next: char, input: &mut Cursor) -> MatcherResult {
		if is_space(next) {
			let mut pos = *input;
			while let Some(next) = input.read() {
				if is_space(next) {
					pos = *input;
				} else {
					break;
				}
			}
			*input = pos;
			MatcherResult::Skip
		} else {
			MatcherResult::None
		}
	}
}
