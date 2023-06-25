use super::*;

pub struct LiteralMatcher;

impl Matcher for LiteralMatcher {
	fn try_match(&self, cursor: &mut Cursor, errors: &mut Errors) -> Option<Node> {
		match cursor.read() {
			Some('\'') => {
				let mut value = String::new();
				loop {
					match cursor.read() {
						Some('\'') => {
							break Some(Node::Literal(value));
						}

						None => {
							errors.add("unclosed string literal");
							break Some(Node::Literal(value));
						}

						Some(char) => value.push(char),
					}
				}
			}

			_ => None,
		}
	}
}
