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
	//----[ Tokens ]----------------------------------------------------------//
	Break,
	Indent(usize),
	Comment,
	Word(Name),
	Symbol(Name),
	Literal(String),
	Integer(u128),
	//----[ Structural ]------------------------------------------------------//
	Module(Span),
	Line(NodeList),
	RawText(Span),
	//----[ AST ]-------------------------------------------------------------//
	Let(Name, NodeList),
}

impl Node {
	pub fn at(self, span: Span) -> NodeData {
		NodeData::new(self, span)
	}

	pub fn name(&self) -> Option<Name> {
		let name = match self {
			Node::Word(name) => name,
			Node::Symbol(name) => name,
			Node::Let(name, ..) => name,
			_ => return None,
		};
		Some(name.clone())
	}
}

/// Encapsulates a [`Node`] with additional data.
#[derive(Clone)]
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

	pub fn offset(&self) -> usize {
		self.span.offset()
	}

	//----------------------------------------------------------------------------------------------------------------//
	// Parsing helpers
	//----------------------------------------------------------------------------------------------------------------//

	pub fn is_symbol(&self, symbol: &str) -> bool {
		if let Node::Symbol(text) = self.get() {
			text == symbol
		} else {
			false
		}
	}

	pub fn is_word(&self, word: &str) -> bool {
		if let Node::Word(text) = self.get() {
			text == word
		} else {
			false
		}
	}

	pub fn name(&self) -> Option<Name> {
		self.get().name()
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

impl PartialEq for NodeData {
	fn eq(&self, other: &Self) -> bool {
		self.node == other.node
	}
}

impl Eq for NodeData {}
