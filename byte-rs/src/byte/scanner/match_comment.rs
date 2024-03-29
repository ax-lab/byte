use super::*;

#[derive(Debug, Eq, PartialEq)]
pub struct Comment;

pub struct CommentMatcher;

impl IsMatcher for CommentMatcher {
	fn try_match(&self, cursor: &mut Span, errors: &mut Errors) -> Option<(Token, Span)> {
		let _ = errors;
		let start = cursor.clone();
		let next = cursor.read();
		match next {
			Some('#') => {
				let (multi, mut level) = if cursor.read_if('(') { (true, 1) } else { (false, 0) };

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

				Some((Token::Comment, cursor.span_from(&start)))
			}

			_ => None,
		}
	}
}
