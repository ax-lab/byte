use crate::Pos;

#[derive(Clone, Debug)]
pub struct Node {
	pub pos: Pos,
	pub end: Pos,
	pub val: NodeValue,
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
