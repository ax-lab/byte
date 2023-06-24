use super::*;

#[derive(Debug, Eq, PartialEq)]
pub struct Integer(pub u128);

has_traits!(Integer: IsNode, Compilable);

impl IsNode for Integer {}

impl Compilable for Integer {
	fn compile(&self, node: &Node, compiler: &Compiler, errors: &mut Errors) -> Option<Expr> {
		let _ = (node, compiler);
		let Integer(value) = self;
		let value = *value;
		if value > IntType::I64.max_value() {
			errors.add("literal value is too big");
			None
		} else {
			let expr = Expr::Value(ValueExpr::Int(IntValue::new(value, IntType::I64)));
			Some(expr)
		}
	}
}

pub struct IntegerMatcher;

impl Matcher for IntegerMatcher {
	fn try_match(&self, cursor: &mut Cursor, errors: &mut Errors) -> Option<Node> {
		let _ = errors;
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
				Some(Node::from(Integer(value)))
			}

			_ => None,
		}
	}
}
