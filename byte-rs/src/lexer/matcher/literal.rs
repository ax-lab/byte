use super::{Cursor, Matcher, MatcherResult, Token};

pub struct MatchLiteral<F: Fn(usize, usize) -> Token>(pub F);

impl<F: Fn(usize, usize) -> Token> Matcher for MatchLiteral<F> {
	fn try_match(&self, next: char, input: &mut Cursor) -> MatcherResult {
		match next {
			'\'' => {
				let pos = input.offset;
				loop {
					let end = input.offset;
					match input.read() {
						Some('\'') => {
							break MatcherResult::Token(self.0(pos, end));
						}

						None => break MatcherResult::Error("unclosed string literal".into()),

						Some(_) => {}
					}
				}
			}

			_ => MatcherResult::None,
		}
	}
}
