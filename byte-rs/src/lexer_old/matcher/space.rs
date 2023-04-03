use crate::core::input::*;

use crate::lexer_old::{Matcher, MatcherResult};

#[derive(Copy, Clone)]
pub struct MatchSpace;

impl Matcher for MatchSpace {
	fn try_match(&self, next: char, input: &mut Cursor) -> MatcherResult {
		if is_space(next) {
			let mut pos = input.clone();
			while let Some(next) = input.read() {
				if is_space(next) {
					pos = input.clone();
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
