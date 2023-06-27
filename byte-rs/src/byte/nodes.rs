use super::*;

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
	Line(Vec<Node>),
	File(Span),
	Module(Span),
	Import(String),
}

impl Node {
	pub fn at(self, span: Span) -> NodeData {
		NodeData::new(self, span)
	}
}

/// Encapsulates a [`Node`] with additional data.
#[derive(Clone, Debug, Eq, PartialEq)]
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

#[derive(Clone)]
pub struct NodeList {
	data: Arc<NodeListData>,
}

impl NodeList {
	pub fn from_single(scope: Handle<Scope>, node: NodeData) -> Self {
		let data = NodeListData {
			scope,
			nodes: vec![node],
		};
		Self { data: data.into() }
	}

	pub fn nodes(&self) -> &[NodeData] {
		&self.data.nodes
	}
}

struct NodeListData {
	scope: Handle<Scope>,
	nodes: Vec<NodeData>,
}
