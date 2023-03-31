use crate::{operator::*, Cursor, Error, Span};

/// Represents a syntactic structure in the source code.
///
/// Nodes either map 1:1 to expressions in the source or are parsed through
/// syntax macros.
///
/// After parsing, a Node is evaluated and the result is the actual compiled
/// program output
#[derive(Clone, Debug)]
pub enum Node {
	None(Cursor),
	Invalid(Error),
	Some(NodeKind, Span),
}

impl Node {
	#[allow(unused)]
	pub fn span(&self) -> Span {
		match self {
			Node::None(cur) => Span {
				pos: *cur,
				end: *cur,
			},
			Node::Invalid(error) => error.span(),
			Node::Some(_, span) => *span,
		}
	}
}

#[derive(Clone, Debug)]
pub enum NodeKind {
	Atom(Atom),
	Unary(OpUnary, Box<NodeKind>),
	Binary(OpBinary, Box<NodeKind>, Box<NodeKind>),
	Ternary(OpTernary, Box<NodeKind>, Box<NodeKind>, Box<NodeKind>),
	Block(Vec<NodeKind>),
	Let(String, Option<Box<NodeKind>>),
	Print(Vec<NodeKind>),
	If {
		expr: Box<NodeKind>,
		block: Box<NodeKind>,
	},
	For {
		id: String,
		from: Box<NodeKind>,
		to: Box<NodeKind>,
		block: Box<NodeKind>,
	},
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
		NodeKind::Atom(self)
	}
}