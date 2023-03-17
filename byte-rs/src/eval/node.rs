use crate::lexer::{Cursor, Span};

#[derive(Clone, Debug)]
pub struct Node<'a> {
	pub span: Span<'a>,
	pub value: NodeValue,
}

impl<'a> Node<'a> {
	pub fn new(pos: Cursor<'a>, end: Cursor<'a>, value: NodeValue) -> Self {
		Node {
			span: Span { pos, end },
			value,
		}
	}
}

#[derive(Clone, Debug)]
pub enum NodeValue {
	None,
	Invalid,
	Atom(Atom),
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
		NodeValue::Atom(self)
	}
}
