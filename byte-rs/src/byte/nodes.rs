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

/// Trait for types that can be used as [`NodeValue`].
pub trait IsNode: IsValue + WithEquality + WithDebug {}

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
pub struct NodeValue {
	id: Id,
	value: Value,
}

impl NodeValue {
	pub fn from<T: IsNode>(value: T) -> Self {
		assert!(get_trait!(&value, IsNode).is_some());
		let id = new_id();
		let value = Value::from(value);
		NodeValue { id, value }
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

	pub fn as_ref(&self) -> NodeRef {
		NodeRef {
			id: self.id,
			value: self.value.as_ref(),
		}
	}
}

impl HasTraits for NodeValue {
	fn type_name(&self) -> &'static str {
		self.value.type_name()
	}

	fn get_trait(&self, type_id: std::any::TypeId) -> Option<&dyn HasTraits> {
		with_trait!(self, type_id, WithEquality);
		with_trait!(self, type_id, WithDebug);
		self.value.get_trait(type_id)
	}
}

impl PartialEq for NodeValue {
	fn eq(&self, other: &Self) -> bool {
		self.value().is_equal(&other.value)
	}
}

impl Eq for NodeValue {}

impl Debug for NodeValue {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		self.value().fmt_debug(f)
	}
}

impl Display for NodeValue {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		self.value().fmt_debug(f)
	}
}

//====================================================================================================================//
// NodeRef
//====================================================================================================================//

#[derive(Clone, Debug)]
pub struct NodeRef {
	id: Id,
	value: ValueRef,
}

impl NodeRef {
	pub fn get(&self) -> NodeValue {
		let id = self.id;
		let value = self.value.get();
		NodeValue { id, value }
	}
}

impl PartialEq for NodeRef {
	fn eq(&self, other: &Self) -> bool {
		self.id == other.id
	}
}

impl Eq for NodeRef {}
