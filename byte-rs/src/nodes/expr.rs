use crate::core::input::*;
use crate::parser::*;

use super::*;

#[derive(Debug)]
pub enum Expr {
	Value(ExprValue),
}

impl IsNode for Expr {}

pub fn parse_expr(_ctx: &mut Context, input: &[ExprItem]) -> Node {
	let sta = input.first().unwrap().span().sta;
	let end = input.last().unwrap().span().end;
	let span = Span { sta, end };
	if input.len() == 1 {
		let value = match &input[0] {
			ExprItem::Token(token) => match token.symbol() {
				Some("true") => ExprValue::Bool { value: true },
				Some("false") => ExprValue::Bool { value: false },
				_ => todo!(),
			},
			_ => todo!(),
		};
		let value = Expr::Value(value);
		Node::new(value, span)
	} else {
		todo!()
	}
}
