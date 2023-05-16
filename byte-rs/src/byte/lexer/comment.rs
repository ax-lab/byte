use std::io::Write;

use crate::core::*;
use crate::lexer::*;
use crate::nodes::*;

#[derive(Eq, PartialEq)]
pub struct Comment(Span);

has_traits!(Comment: IsNode, WithEquality);

impl IsNode for Comment {}

impl HasRepr for Comment {
	fn output_repr(&self, output: &mut Repr<'_>) -> std::io::Result<()> {
		if output.is_debug() {
			write!(output, "Comment({})", self.0)
		} else {
			write!(output, "comment")
		}
	}
}

pub struct CommentMatcher;

impl Matcher for CommentMatcher {
	fn try_match(&self, cursor: &mut Cursor, _errors: &mut Errors) -> Option<Node> {
		let start = cursor.clone();
		let next = cursor.read();
		match next {
			Some('#') => {
				let (multi, mut level) = if cursor.try_read('(') {
					(true, 1)
				} else {
					(false, 0)
				};

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

				let span = Span::from(&start, cursor);
				Some(Node::from(Comment(span)))
			}

			_ => None,
		}
	}
}
