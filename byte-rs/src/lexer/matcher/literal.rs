use crate::lexer::LexerError;

use crate::input::*;

use super::{Matcher, MatcherResult, Token};

#[derive(Copy, Clone)]
pub struct MatchLiteral;

impl Matcher for MatchLiteral {
	fn try_match(&self, next: char, input: &mut Cursor) -> MatcherResult {
		match next {
			'\'' => {
				let pos = *input;
				loop {
					let end = *input;
					match input.read() {
						Some('\'') => {
							break MatcherResult::Token(Token::Literal(
								input.src().text(Span { sta: pos, end }),
							));
						}

						None => break MatcherResult::Error(LexerError::UnclosedLiteral),

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
