use crate::{
	lexer::{Cursor, Span},
	Error,
};

use super::{Context, OpBinary, OpTernary, OpUnary};

#[derive(Clone, Debug)]
pub struct Node<'a> {
	pub value: NodeValue,
	pub span: Span<'a>,
}

impl<'a> Node<'a> {
	pub fn new(pos: Cursor<'a>, end: Cursor<'a>, value: NodeValue) -> Self {
		Node {
			span: Span { pos, end },
			value,
		}
	}

	pub fn as_expression(self, context: &mut Context<'a>) -> Result<Expr, Node<'a>> {
		match self.value {
			NodeValue::Expr(expr) => Ok(expr),
			NodeValue::Invalid => Err(self),
			NodeValue::None => {
				context.add_error(Error::ExpectedExpression(context.span()));
				Err(NodeValue::Invalid.at_pos(context.pos()))
			}
		}
	}
}

#[derive(Clone, Debug)]
pub enum NodeValue {
	None,
	Invalid,
	Expr(Expr),
}

impl NodeValue {
	pub fn at<'a>(self, pos: Cursor<'a>, end: Cursor<'a>) -> Node<'a> {
		Node {
			span: Span { pos, end },
			value: self,
		}
	}

	pub fn at_pos<'a>(self, pos: Cursor<'a>) -> Node<'a> {
		Node {
			span: Span { pos, end: pos },
			value: self,
		}
	}

	pub fn at_span<'a>(self, span: Span<'a>) -> Node<'a> {
		Node { span, value: self }
	}
}

#[derive(Clone, Debug)]
pub enum Expr {
	Value(Atom),
	Unary(OpUnary, Box<Expr>),
	Binary(OpBinary, Box<Expr>, Box<Expr>),
	Ternary(OpTernary, Box<Expr>, Box<Expr>, Box<Expr>),
}

#[derive(Clone, Debug)]
pub enum Atom {
	Null,
	Bool(bool),
	String(String),
	Integer(u64),
	Id(String),
}

impl Atom {
	pub fn as_value(self) -> NodeValue {
		NodeValue::Expr(Expr::Value(self))
	}
}
