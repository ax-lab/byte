use super::{Cursor, Matcher, MatcherResult, Token};

#[derive(Copy, Clone)]
pub struct MatchLiteral;

impl Matcher for MatchLiteral {
	fn try_match(&self, next: char, input: &mut Cursor) -> MatcherResult {
		match next {
			'\'' => {
				let pos = input.offset;
				loop {
					let end = input.offset;
					match input.read() {
						Some('\'') => {
							break MatcherResult::Token(Token::Literal(pos, end));
						}

						None => break MatcherResult::Error("unclosed string literal".into()),

						Some(_) => {}
					}
				}
			}

			_ => MatcherResult::None,
		}
	}

	fn clone_box(&self) -> Box<dyn Matcher> {
		Box::new(self.clone())
	}
}
