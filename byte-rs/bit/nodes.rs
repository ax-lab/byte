use crate::{Source, Symbol};

pub enum Key<'a> {
	None,
	Source,
	Symbol(Symbol<'a>),
	LineBreak,
	Value,
}

pub enum Node<'a> {
	None,
	Source(Source<'a>),
	Symbol(Symbol<'a>),
	LineBreak,
	Int(i32),
}

impl<'a> Node<'a> {
	pub fn key(&self) -> Key<'a> {
		match self {
			Node::None => Key::None,
			Node::Source(..) => Key::Source,
			Node::Symbol(symbol) => Key::Symbol(*symbol),
			Node::LineBreak => Key::LineBreak,
			Node::Int(..) => Key::Value,
		}
	}
}
