use crate::core::*;
use crate::lexer::*;
use crate::nodes::*;

pub struct IdentifierMatcher;

impl Matcher for IdentifierMatcher {
	fn try_match(&self, cursor: &mut Cursor, _errors: &mut Errors) -> Option<Node> {
		let start = cursor.clone();
		let next = cursor.read();
		match next {
			Some('a'..='z' | 'A'..='Z' | '_') => {
				let mut pos;
				loop {
					pos = cursor.clone();
					match cursor.read() {
						Some('a'..='z' | 'A'..='Z' | '_' | '0'..='9') => {}
						_ => {
							*cursor = pos;
							break;
						}
					}
				}

				let span = Span::from(&start, cursor);
				Some(Node::from(Token::Word(span)))
			}

			_ => None,
		}
	}
}

impl NodeStream {
	pub fn read_id(&mut self) -> Option<Node> {
		self.read_if(|n| matches!(n.get::<Token>(), Some(Token::Word(..))))
	}
}
