use crate::core::input::*;

use super::{Matcher, MatcherResult, Token};

#[derive(Copy, Clone)]
pub struct MatchNumber;

impl Matcher for MatchNumber {
	fn try_match(&self, next: char, input: &mut Cursor) -> MatcherResult {
		match next {
			'0'..='9' => {
				let mut value = decimal_value(next);
				let mut pos;
				loop {
					pos = *input;
					match input.read() {
						Some(next @ '0'..='9') => {
							value = value * 10 + decimal_value(next);
						}
						_ => {
							break;
						}
					}
				}
				*input = pos;
				MatcherResult::Token(Token::Integer(value))
			}

			_ => MatcherResult::None,
		}
	}

	fn clone_box(&self) -> Box<dyn Matcher> {
		Box::new(self.clone())
	}
}

fn decimal_value(n: char) -> u64 {
	(n as u64) - ('0' as u64)
}
