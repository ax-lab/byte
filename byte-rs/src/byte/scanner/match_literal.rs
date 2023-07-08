use super::*;

pub struct LiteralMatcher;

impl IsMatcher for LiteralMatcher {
	fn try_match(&self, cursor: &mut Span, errors: &mut Errors) -> Option<(Token, Span)> {
		let start = cursor.clone();
		match cursor.read() {
			Some('\'') => {
				let mut value = String::new();
				loop {
					match cursor.read() {
						Some('\'') => {
							break Some((Token::Literal(value.into()), cursor.span_from(&start)));
						}

						None => {
							let span = cursor.span_from(&start);
							errors.add("unclosed string literal", span.clone());
							break Some((Token::Literal(value.into()), span));
						}

						Some(char) => value.push(char),
					}
				}
			}

			_ => None,
		}
	}
}
