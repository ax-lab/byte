use super::*;

#[derive(Debug, Eq, PartialEq)]
pub struct Integer(pub u128);

has_traits!(Integer: IsNode, Compilable);

impl IsNode for Integer {
	// TODO: this whole precedence and evaluate complete need to be better defined and simplified

	fn precedence(&self, context: &Context) -> Option<(Precedence, Sequence)> {
		let _ = context;
		Some((Precedence::Values, Sequence::AtOnce))
	}

	fn evaluate(&self, context: &mut EvalContext) -> Result<NodeEval> {
		let _ = context;
		Ok(NodeEval::Complete)
	}
}

impl Compilable for Integer {
	fn compile(&self, node: &Node, context: &Context, errors: &mut Errors) -> Option<Expr> {
		let _ = context;
		let Integer(value) = self;
		let value = *value;
		if value > IntType::I64.max_value() {
			errors.add_at("literal value is too big", node.span().cloned());
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
