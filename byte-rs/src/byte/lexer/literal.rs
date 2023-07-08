use super::*;

pub struct LiteralMatcher;

impl IsMatcher for LiteralMatcher {
	fn try_match(&self, cursor: &mut Span, errors: &mut Errors) -> Option<Node> {
		let start = cursor.clone();
		match cursor.read() {
			Some('\'') => {
				let mut value = String::new();
				loop {
					match cursor.read() {
						Some('\'') => {
							break Some(Bit::Literal(value).at(cursor.span_from(&start)));
						}

						None => {
							let span = cursor.span_from(&start);
							errors.add("unclosed string literal", span.clone());
							break Some(Bit::Literal(value).at(span));
						}

						Some(char) => value.push(char),
					}
				}
			}

			_ => None,
		}
	}
}
