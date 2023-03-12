use super::{Cursor, Matcher, MatcherResult, Token};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct LexString {
	pub pos: usize,
	pub end: usize,
}

pub struct LexLiteral<F: Fn(LexString) -> Token>(pub F);

impl<F: Fn(LexString) -> Token> Matcher for LexLiteral<F> {
	fn try_match(&self, next: char, input: &mut Cursor) -> MatcherResult {
		match next {
			'\'' => {
				let pos = input.offset;
				loop {
					let end = input.offset;
					match input.read() {
						Some('\'') => {
							let str = LexString { pos, end };
							break MatcherResult::Token(self.0(str));
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
