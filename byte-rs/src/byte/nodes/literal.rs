use super::*;

#[derive(Debug, Eq, PartialEq)]
pub struct Literal(pub String);

has_traits!(Literal: IsNode, Compilable);

impl IsNode for Literal {}

impl Compilable for Literal {
	fn compile(&self, node: &Node, compiler: &Compiler, errors: &mut Errors) -> Option<Expr> {
		let _ = (node, errors);
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
				let mut value = String::new();
				loop {
					match cursor.read() {
						Some('\'') => {
							break Some(Node::from(Literal(value)));
						}

						None => {
							errors.add("unclosed string literal");
							break Some(Node::from(Literal(value)));
						}

						Some(char) => value.push(char),
					}
				}
			}

			_ => None,
		}
	}
}
