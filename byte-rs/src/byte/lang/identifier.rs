use std::io::Write;

use crate::core::*;
use crate::lexer::*;
use crate::nodes::*;

#[derive(Clone, Eq, PartialEq)]
pub struct Identifier(Span);

has_traits!(Identifier: IsNode, WithEquality);

impl Identifier {
	pub fn value(&self) -> &str {
		self.0.text()
	}
}

impl IsNode for Identifier {}

impl HasRepr for Identifier {
	fn output_repr(&self, output: &mut Repr<'_>) -> std::io::Result<()> {
		let text = self.0.text();
		if output.is_debug() {
			write!(output, "Identifier({text:?})")
		} else {
			write!(output, "`{text}`")
		}
	}
}

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

				let span = Span::new(&start, cursor);
				Some(Node::from(Identifier(span)))
			}

			_ => None,
		}
	}
}

impl Node {
	pub fn get_identifier(&self) -> Option<&Identifier> {
		self.get::<Identifier>()
	}
}
