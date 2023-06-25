use super::*;

/// Trait for types that can be used as [`NodeValue`].
pub trait IsNode: IsValue + WithEquality + WithDebug {}

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
	Import(String),
}

#[derive(Clone)]
pub struct NodeList {
	data: Arc<NodeListData>,
}

struct NodeListData {
	scope: Arc<ScopeData>,
	nodes: Vec<Node>,
	changes: RwLock<NodeChange>,
}

struct NodeChange {
	index: usize,
	count: usize,
	nodes: Vec<Node>,
}
