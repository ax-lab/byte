use std::io::Write;

use crate::core::*;
use crate::lexer::*;
use crate::nodes::*;

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Integer(pub u128);

impl IsNode for Integer {}

has_traits!(Integer: IsNode, WithEquality);

impl HasRepr for Integer {
	fn output_repr(&self, output: &mut Repr<'_>) -> std::io::Result<()> {
		if output.is_debug() {
			write!(output, "Integer({})", self.0)
		} else {
			write!(output, "{}", self.0)
		}
	}
}

fmt_from_repr!(Integer);

pub struct IntegerMatcher;

impl Matcher for IntegerMatcher {
	fn try_match(&self, cursor: &mut Cursor, _errors: &mut Errors) -> Option<Node> {
		match cursor.read() {
			Some(next @ '0'..='9') => {
				let mut value = decimal_value(next);
				let mut pos;
				loop {
					pos = cursor.clone();
					match cursor.read() {
						Some(next @ '0'..='9') => {
							value = value * 10 + decimal_value(next);
						}
						_ => {
							break;
						}
					}
				}
				*cursor = pos;
				Some(Node::from(Integer(value)))
			}

			_ => None,
		}
	}
}

impl Node {
	pub fn get_integer(&self) -> Option<Integer> {
		self.get::<Integer>().cloned()
	}
}

fn decimal_value(n: char) -> u128 {
	(n as u128) - ('0' as u128)
}
