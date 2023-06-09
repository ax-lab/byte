pub mod raw;
pub mod token;

pub use raw::*;
pub use token::*;

use std::{
	collections::{HashMap, HashSet},
	ops::{Index, RangeBounds},
};

use super::*;

/// Trait for types that can be used as [`Node`].
pub trait IsNode: IsValue + WithEquality + WithDebug {
	fn get_bindings(&self) -> Vec<Name> {
		Vec::new()
	}

	/// Return this node's evaluation precedence if it can be solved by the
	/// given context.
	///
	/// Return [`None`] to indicate the node cannot be currently evaluated.
	fn precedence(&self, context: &Context) -> Option<(Precedence, Sequence)>;

	fn evaluate(&self, context: &mut EvalContext) -> Result<NodeEval>;
}

pub enum NodeEval {
	None,
	Complete,
}

//====================================================================================================================//
// Node
//====================================================================================================================//

/// Stores an immutable parsing node.
///
/// Nodes are at the core of the language parsing and evaluation. They can
/// range from unparsed raw text nodes, token lists, AST nodes, all the way
/// to fully resolved semantic nodes.
///
/// Nodes are resolved within a [`Context`] generating a new list of nodes
/// and a set of changes to apply to the context. The process is repeated
/// until all nodes are fully resolved.
///
/// Resolved nodes are used to generate [`code`], which can then be run or
/// compiled to an executable format.
#[derive(Clone)]
pub struct Node {
	id: Id,
	value: Value,
	span: Option<Span>,
}

impl Node {
	pub fn from<T: IsNode>(value: T, span: Option<Span>) -> Self {
		assert!(get_trait!(&value, IsNode).is_some());
		let id = new_id();
		let value = Value::from(value);
		Node { id, value, span }
	}

	/// Globally unique ID for this node.
	pub fn id(&self) -> Id {
		self.id
	}

	/// Attempt to downcast the node value to a concrete [`IsNode`] type.
	pub fn get<T: IsNode>(&self) -> Option<&T> {
		self.value.get()
	}

	/// Reference to the inner [`IsNode`] value.
	pub fn value(&self) -> &dyn IsNode {
		get_trait!(self, IsNode).unwrap()
	}

	/// Reference to the inner node value as an [`IsValue`].
	pub fn as_value(&self) -> &dyn IsValue {
		self.value.as_value()
	}

	/// Source span for this node if available.
	pub fn span(&self) -> Option<&Span> {
		self.span.as_ref().or_else(|| self.value.span())
	}
}

impl HasTraits for Node {
	fn type_name(&self) -> &'static str {
		self.value.type_name()
	}

	fn get_trait(&self, type_id: std::any::TypeId) -> Option<&dyn HasTraits> {
		with_trait!(self, type_id, WithEquality);
		with_trait!(self, type_id, WithDebug);
		self.value.get_trait(type_id)
	}
}

impl PartialEq for Node {
	fn eq(&self, other: &Self) -> bool {
		self.value().is_equal(&other.value)
	}
}

impl Eq for Node {}

impl Debug for Node {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		self.value().fmt_debug(f)
	}
}

impl Display for Node {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		self.value().fmt_debug(f)
	}
}

impl WithSpan for Node {
	fn span(&self) -> Option<&Span> {
		Node::span(self)
	}
}

//====================================================================================================================//
// NodeSet
//====================================================================================================================//

/// Maintains a set of [`Node`] for evaluation.
///
/// Nodes in the set are strictly sorted by their [`Id`], without duplicates.
#[derive(Default, Clone)]
pub struct NodeSet {
	nodes: Arc<Vec<Node>>,
}

impl NodeSet {
	pub fn empty() -> Self {
		Self {
			nodes: Default::default(),
		}
	}

	pub fn single(node: Node) -> Self {
		NodeSet {
			nodes: Arc::new(vec![node]),
		}
	}

	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}

	pub fn len(&self) -> usize {
		self.nodes.len()
	}

	pub fn include(&self, node: Node) -> NodeSet {
		if self.len() == 0 {
			NodeSet::single(node)
		} else {
			self.extend(std::iter::once(node))
		}
	}

	pub fn combine(&self, other: &NodeSet) -> NodeSet {
		if self.len() == 0 {
			other.clone()
		} else if other.len() == 0 {
			self.clone()
		} else {
			self.extend(other.iter().cloned())
		}
	}

	pub fn subtract(&self, other: &NodeSet) -> NodeSet {
		if self.len() == 0 || other.len() == 0 {
			self.clone()
		} else {
			let other: HashSet<Id> = other.iter().map(|x| x.id()).collect();
			let nodes = self.iter().filter(|x| !other.contains(&x.id())).cloned();
			NodeSet {
				nodes: Arc::new(nodes.collect()),
			}
		}
	}

	pub fn extend<'a, T: IntoIterator<Item = Node>>(&self, nodes: T) -> NodeSet {
		let nodes = nodes.into_iter();
		let nodes = self.nodes.iter().cloned().chain(nodes);
		let nodes: HashMap<Id, Node> = nodes.map(|x| (x.id(), x)).collect();
		let mut nodes: Vec<Node> = nodes.into_values().collect();
		nodes.sort_by_key(|x| x.id());
		NodeSet { nodes: Arc::new(nodes) }
	}

	pub fn range<T: RangeBounds<usize>>(&self, range: T) -> NodeSet {
		let range = compute_range(range, self.len());
		let range = &self.nodes[range];
		if range.len() == self.len() {
			self.clone()
		} else {
			NodeSet {
				nodes: range.to_vec().into(),
			}
		}
	}

	pub fn contains(&self, node: &Node) -> bool {
		self.index_of(node).is_some()
	}

	pub fn index_of(&self, node: &Node) -> Option<usize> {
		if let Ok(index) = self.nodes.binary_search_by_key(&node.id(), |it| it.id()) {
			Some(index)
		} else {
			None
		}
	}

	pub fn iter(&self) -> NodeSetIterator {
		NodeSetIterator { set: self, index: 0 }
	}
}

impl Index<usize> for NodeSet {
	type Output = Node;

	fn index(&self, index: usize) -> &Self::Output {
		&self.nodes[index]
	}
}

/// Iterator for a [`NodeSet`].
pub struct NodeSetIterator<'a> {
	set: &'a NodeSet,
	index: usize,
}

impl<'a> Iterator for NodeSetIterator<'a> {
	type Item = &'a Node;

	fn next(&mut self) -> Option<Self::Item> {
		let index = self.index;
		if index < self.set.len() {
			self.index += 1;
			Some(&self.set.nodes[index])
		} else {
			None
		}
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		let remaining = self.set.len() - self.index;
		(remaining, Some(remaining))
	}
}
