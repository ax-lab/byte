pub mod comment;
pub mod literal;
pub mod number;
pub mod raw;
pub mod token;

pub use comment::*;
pub use literal::*;
pub use number::*;
pub use raw::*;
pub use token::*;

use std::ops::{Index, RangeBounds};

use super::code::*;
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

#[derive(Copy, Clone, Eq, PartialEq)]
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
// NodeList
//====================================================================================================================//

/// Maintains a list of [`Node`] for evaluation.
#[derive(Default, Clone)]
pub struct NodeList {
	nodes: Arc<Vec<Node>>,
}

impl NodeList {
	pub fn new<T: IntoIterator<Item = Node>>(items: T) -> Self {
		let nodes: Vec<Node> = items.into_iter().collect();
		Self { nodes: nodes.into() }
	}

	pub fn empty() -> Self {
		Self {
			nodes: Default::default(),
		}
	}

	pub fn single(node: Node) -> Self {
		NodeList {
			nodes: Arc::new(vec![node]),
		}
	}

	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}

	pub fn span(&self) -> Option<Span> {
		self.nodes.first().and_then(|x| x.span().cloned())
	}

	pub fn len(&self) -> usize {
		self.nodes.len()
	}

	pub fn range<T: RangeBounds<usize>>(&self, range: T) -> NodeList {
		let range = compute_range(range, self.len());
		let range = &self.nodes[range];
		if range.len() == self.len() {
			self.clone()
		} else {
			NodeList {
				nodes: range.to_vec().into(),
			}
		}
	}

	pub fn slice<T: RangeBounds<usize>>(&self, range: T) -> &[Node] {
		let range = compute_range(range, self.len());
		&self.nodes[range]
	}

	pub fn iter(&self) -> NodeListIterator {
		NodeListIterator { list: self, index: 0 }
	}

	pub fn as_slice(&self) -> &[Node] {
		self.nodes.as_slice()
	}
}

impl Index<usize> for NodeList {
	type Output = Node;

	fn index(&self, index: usize) -> &Self::Output {
		&self.nodes[index]
	}
}

/// Iterator for a [`NodeList`].
pub struct NodeListIterator<'a> {
	list: &'a NodeList,
	index: usize,
}

impl<'a> Iterator for NodeListIterator<'a> {
	type Item = &'a Node;

	fn next(&mut self) -> Option<Self::Item> {
		let index = self.index;
		if index < self.list.len() {
			self.index += 1;
			Some(&self.list.nodes[index])
		} else {
			None
		}
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		let remaining = self.list.len() - self.index;
		(remaining, Some(remaining))
	}
}

impl Debug for NodeList {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		write!(f, "<NodeList")?;
		for (n, it) in self.iter().enumerate() {
			let mut f = f.indented();
			write!(f, "\n>>> [{n}]")?;
			if let Some(span) = it.span() {
				span.format_full(" at ", &mut f)?;
				write!(f, "    # {:?}", it.id())?;
			}
			write!(f, "")?;
			write!(f.indented_with("... "), "\n{it:?}")?;
		}
		write!(f, "\n>")
	}
}
