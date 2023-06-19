pub mod comment;
pub mod line;
pub mod literal;
pub mod node_list;
pub mod number;
pub mod raw;
pub mod tokens;

pub use comment::*;
pub use line::*;
pub use literal::*;
pub use node_list::*;
pub use number::*;
pub use raw::*;
pub use tokens::*;

use std::ops::{Index, RangeBounds};

use super::code::*;
use super::*;

/// Trait for types that can be used as [`Node`].
pub trait IsNode: IsValue + WithEquality + WithDebug {
	fn precedence(&self) -> Option<(Precedence, Sequence)> {
		None
	}

	fn evaluate(&self, context: &mut ResolveContext) {
		let _ = context;
	}
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

	/// Return true if the node value is of the given type.
	pub fn is<T: IsNode>(&self) -> bool {
		self.get::<T>().is_some()
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
