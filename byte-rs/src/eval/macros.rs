use super::{parser, Atom, Context, Expr, Node, NodeValue, OpBinary};

pub trait Macro {
	fn parse<'a>(&self, context: &mut Context<'a>) -> Node<'a>;
}

pub struct AddOne;

impl Macro for AddOne {
	fn parse<'a>(&self, context: &mut Context<'a>) -> Node<'a> {
		let node = parser::parse_expression(context);
		let span = node.span;
		match node.as_expression(context) {
			Ok(expr) => {
				let expr = Expr::Binary(
					OpBinary::Add,
					Box::new(expr),
					Box::new(Expr::Value(Atom::Integer(1))),
				);
				let value = NodeValue::Expr(expr);
				value.at_span(span)
			}
			Err(node) => node,
		}
	}
}
