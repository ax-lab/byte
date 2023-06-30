use super::*;

pub mod list;

pub use list::*;

/// Enumeration of all available language nodes.
///
/// Nodes relate to the source code, representing language constructs of all
/// levels, from files, raw text, and tokens, all the way to fully fledged
/// definitions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Node {
	Break,
	Indent(usize),
	Comment,
	Word(Name),
	Symbol(Name),
	Literal(String),
	Integer(u128),
	RawText(Span),
	Module(Span),
}

impl Node {
	pub fn at(self, span: Span) -> NodeData {
		NodeData::new(self, span)
	}
}

/// Encapsulates a [`Node`] with additional data.
#[derive(Clone, Eq, PartialEq)]
pub struct NodeData {
	id: Id,
	node: Node,
	span: Span,
}

impl NodeData {
	pub fn new(node: Node, span: Span) -> Self {
		let id = new_id();
		Self { id, node, span }
	}

	pub fn id(&self) -> Id {
		self.id
	}

	pub fn get(&self) -> &Node {
		&self.node
	}

	pub fn to_inner(self) -> Node {
		self.node
	}

	pub fn span(&self) -> &Span {
		&self.span
	}
}

impl Display for NodeData {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:?}", self.node)
	}
}

impl Debug for NodeData {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		let (id, node) = (self.id, &self.node);
		write!(f, "<{id} {node:?}>")
	}
}
