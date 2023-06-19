use super::*;

#[derive(Debug, Eq, PartialEq)]
pub struct Comment;

has_traits!(Comment: IsNode);

impl IsNode for Comment {}

pub struct CommentMatcher;

impl Matcher for CommentMatcher {
	fn try_match(&self, cursor: &mut Cursor, errors: &mut Errors) -> Option<Node> {
		let _ = errors;
		let start = cursor.clone();
		let next = cursor.read();
		match next {
			Some('#') => {
				let (multi, mut level) = if cursor.try_read('(') { (true, 1) } else { (false, 0) };

				let mut pos;
				let putback = loop {
					pos = cursor.clone();
					match cursor.read() {
						Some('\n' | '\r') if !multi => break true,
						Some('(') if multi => {
							level += 1;
						}
						Some(')') if multi => {
							level -= 1;
							if level == 0 {
								break false;
							}
						}
						Some(_) => {}
						None => break false,
					}
				};
				if putback {
					*cursor = pos;
				}

				let span = cursor.span_from(&start);
				Some(Node::from(Comment, Some(span)))
			}

			_ => None,
		}
	}
}
