use super::*;

#[derive(Debug, Eq, PartialEq)]
pub struct Integer(pub u128);

has_traits!(Integer: IsNode);

impl IsNode for Integer {
	fn precedence(&self, context: &Context) -> Option<(Precedence, Sequence)> {
		let _ = context;
		todo!()
	}

	fn evaluate(&self, context: &mut EvalContext) -> Result<NodeEval> {
		let _ = context;
		todo!()
	}
}

pub struct IntegerMatcher;

impl Matcher for IntegerMatcher {
	fn try_match(&self, cursor: &mut Cursor, errors: &mut Errors) -> Option<Node> {
		let _ = errors;
		let start = cursor.clone();
		match cursor.read() {
			Some(next @ '0'..='9') => {
				let mut value = digit_value(next);
				let mut pos;
				loop {
					pos = cursor.clone();
					match cursor.read() {
						Some(next @ '0'..='9') => {
							value = value * 10 + digit_value(next);
						}
						_ => {
							break;
						}
					}
				}
				*cursor = pos;
				let span = cursor.span_from(&start);
				Some(Node::from(Integer(value), Some(span)))
			}

			_ => None,
		}
	}
}
