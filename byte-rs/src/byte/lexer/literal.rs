use super::*;

pub struct LiteralMatcher;

impl Matcher for LiteralMatcher {
	fn try_match(&self, cursor: &mut Span, errors: &mut Errors) -> Option<NodeData> {
		let start = cursor.clone();
		match cursor.read() {
			Some('\'') => {
				let mut value = String::new();
				loop {
					match cursor.read() {
						Some('\'') => {
							break Some(Node::Literal(value).at(cursor.span_from(&start)));
						}

						None => {
							let span = cursor.span_from(&start);
							errors.add_at("unclosed string literal", span.clone());
							break Some(Node::Literal(value).at(span));
						}

						Some(char) => value.push(char),
					}
				}
			}

			_ => None,
		}
	}
}
