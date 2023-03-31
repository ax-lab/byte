use crate::{
	input::is_space,
	lexer::{Matcher, MatcherResult},
	Cursor,
};

#[derive(Copy, Clone)]
pub struct MatchSpace;

impl Matcher for MatchSpace {
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

	fn clone_box(&self) -> Box<dyn Matcher> {
		Box::new(self.clone())
	}
}
