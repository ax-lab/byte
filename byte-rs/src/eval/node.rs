use crate::{
	lexer::{Cursor, Span},
	Error,
};

use super::{Context, OpBinary, OpTernary, OpUnary};

/// Represents a syntactic structure in the source code.
///
/// Nodes either map 1:1 to expressions in the source or are parsed through
/// syntax macros.
///
/// After parsing, a Node is evaluated and the result is the actual compiled
/// program output
#[derive(Clone, Debug)]
pub struct Node<'a> {
	pub value: NodeKind,
	pub span: Span<'a>,
}

impl<'a> Node<'a> {
	pub fn new(pos: Cursor<'a>, end: Cursor<'a>, value: NodeKind) -> Self {
		Node {
			span: Span { pos, end },
			value,
		}
	}

	#[allow(unused)]
	pub fn as_expression(self, context: &mut Context<'a>) -> Result<Expr, Node<'a>> {
		match self.value {
			NodeKind::Expr(expr) => Ok(expr),
			NodeKind::Invalid => Err(self),
			NodeKind::None => {
				context.add_error(Error::ExpectedExpression(context.span()));
				Err(NodeKind::Invalid.at_pos(context.pos()))
			}
		}
	}
}

#[derive(Clone, Debug)]
pub enum NodeKind {
	None,
	Invalid,
	Expr(Expr),
}

impl NodeKind {
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

	#[allow(unused)]
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
	Let(String, Option<Box<Expr>>),
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
	pub fn as_value(self) -> NodeKind {
		NodeKind::Expr(Expr::Value(self))
	}
}
