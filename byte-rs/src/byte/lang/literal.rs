use std::io::Write;

use crate::core::*;
use crate::lexer::*;
use crate::nodes::*;

#[derive(Clone, Eq, PartialEq)]
pub struct Literal(String);

has_traits!(Literal: IsNode, WithEquality);

impl Literal {
	pub fn as_str(&self) -> &str {
		self.0.as_str()
	}
}

impl IsNode for Literal {}

impl HasRepr for Literal {
	fn output_repr(&self, output: &mut Repr<'_>) -> std::io::Result<()> {
		if output.is_debug() {
			write!(output, "Literal({:?})", self.as_str())
		} else {
			write!(output, "{:?}", self.as_str())
		}
	}
}

impl AsRef<str> for Literal {
	fn as_ref(&self) -> &str {
		self.as_str()
	}
}

pub struct LiteralMatcher;

impl Matcher for LiteralMatcher {
	fn try_match(&self, cursor: &mut Cursor, errors: &mut Errors) -> Option<Node> {
		match cursor.read() {
			Some('\'') => {
				let pos = cursor.clone();
				loop {
					let end = cursor.clone();
					match cursor.read() {
						Some('\'') => {
							let span = Span::from(&pos, &end);
							let value = span.text().to_string();
							break Some(Node::from(Literal(value)));
						}

						None => {
							let span = Span::from(&pos, &end);
							let value = span.text().to_string();
							errors.add("unclosed string literal".at(span));
							break Some(Node::from(Literal(value)));
						}

						Some(_) => {}
					}
				}
			}

			_ => None,
		}
	}
}
