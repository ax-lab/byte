use super::{Cursor, Matcher, MatcherResult, Token};

pub struct MatchNumber<F: Fn(u64) -> Token>(pub F);

impl<F: Fn(u64) -> Token> Matcher for MatchNumber<F> {
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
				MatcherResult::Token(self.0(value))
			}

			_ => MatcherResult::None,
		}
	}
}

fn decimal_value(n: char) -> u64 {
	(n as u64) - ('0' as u64)
}
