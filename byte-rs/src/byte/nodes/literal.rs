use super::*;

#[derive(Debug, Eq, PartialEq)]
pub struct Literal(pub String);

has_traits!(Literal: IsNode, Compilable);

impl IsNode for Literal {
	fn precedence(&self, context: &Context) -> Option<(Precedence, Sequence)> {
		let _ = context;
		Some((Precedence::Values, Sequence::AtOnce))
	}

	fn evaluate(&self, context: &mut EvalContext) -> Result<NodeEval> {
		let _ = context;
		Ok(NodeEval::Complete)
	}
}

impl Compilable for Literal {
	fn compile(&self, node: &Node, context: &Context, errors: &mut Errors) -> Option<Expr> {
		let _ = (node, errors);
		let compiler = context.compiler();
		let str = StrValue::new(self.as_str(), &compiler);
		Some(Expr::Value(ValueExpr::Str(str)))
	}
}

impl Literal {
	pub fn as_str(&self) -> &str {
		self.0.as_str()
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
							let span = end.span_from(&pos);
							let value = span.text().to_string();
							break Some(Node::from(Literal(value), Some(span)));
						}

						None => {
							let span = end.span_from(&pos);
							let value = span.text().to_string();
							errors.add_at("unclosed string literal", Some(span.clone()));
							break Some(Node::from(Literal(value), Some(span)));
						}

						Some(_) => {}
					}
				}
			}

			_ => None,
		}
	}
}
