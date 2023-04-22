use std::any::{Any, TypeId};
use std::cell::{Ref, RefCell};
use std::collections::VecDeque;
use std::fmt::{Debug, Display};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{self, AtomicU64};
use std::sync::{Arc, Mutex, MutexGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::core::error::*;
use crate::core::input::*;
use crate::core::repr::*;
use crate::core::str::*;
use crate::core::*;
use crate::vm::expr::Expr;
use crate::vm::operators::*;

use super::*;

/// Represents parsed content from a source code input. A node can range from
/// an abstract blob of tokens to a fully analyzed semantic expression.
///
/// The [`Node`] is just a generic container, with the actual node value being
/// given by an underlying [`IsNode`] value.
///
/// Note that node cloning is shallow: cloned instances share the same value,
/// seeing any changes to that value, and retain the same id.
///
/// `IsNode` evaluation
/// -------------------
///
/// At the very minimum, the `IsNode` implementation must provide an `eval`
/// method returning a [`NodeEval`] result.
///
/// As a node is evaluated, the underlying `IsNode` value can change into other
/// intermediate values. This intermediate value will continue being evaluated
/// until a fully resolved value.
///
/// The above node replacement process is what allows for macros, expansions,
/// and substitutions.
///
/// Nodes and expressions
/// ---------------------
///
/// Resolved nodes are expected to be resolvable into an [`Expr`] which can
/// then be executed or compiled to binary code.
///
/// Not all resolved nodes need to be an `Expr` however. Intermediate nodes
/// may serve specific purposes and be used only as child of other nodes.
///
/// To account for the variety of nodes and contexts, `IsNode` implements
/// the [`HasTraits`] trait to provide for specific functionality.
///
/// Specific node traits follow the pattern of `Is(SomeNodeAspect)Node`. For
/// example, the [`IsExprValueNode`] trait.
#[derive(Clone)]
pub struct Node {
	id: u64,
	node: Arc<RwLock<Value>>,
	span: Arc<RwLock<Option<Span>>>,
	done: Arc<RwLock<bool>>,
}

impl PartialEq for Node {
	fn eq(&self, other: &Self) -> bool {
		self.id == other.id
	}
}

/// Root trait implemented for a [`Node`] underlying value.
pub trait IsNode: IsValue + HasRepr {
	fn eval(&mut self, errors: &mut ErrorList) -> NodeEval;

	fn span(&self) -> Option<Span> {
		None
	}
}

/// Implemented by nodes which can possibly be used in an expression value
/// context.
pub trait IsExprValueNode {
	/// Returns if this node can be used as a value in an expression.
	///
	/// The node may return [`None`] if it's unresolved and needs to wait
	/// to determine if it's a value or not.
	fn is_value(&self) -> Option<bool>;
}

/// Implemented by nodes which can resolve to an operand in an expression
/// context.
pub trait IsOperatorNode {
	/// Return the corresponding unary operator if this is a valid
	/// prefix unary operator symbol.
	fn get_unary_pre(&self) -> Option<OpUnary>;

	/// Return the corresponding unary operator if this is a valid
	/// posfix unary operator symbol.
	fn get_unary_pos(&self) -> Option<OpUnary>;

	/// Return the corresponding binary operator if this is a valid
	/// binary operator symbol.
	fn get_binary(&self) -> Option<OpBinary>;

	/// Return the corresponding ternary operator and delimiter symbol
	/// if this is a valid ternary operator symbol.
	fn get_ternary(&self) -> Option<(OpTernary, &'static str)>;
}

impl Node {
	pub fn new<T: IsNode>(node: T) -> Self {
		// Generate a unique ID for each instance. The ID will remain constant
		// even if the underlying value changes and is preserved by cloning.
		static ID: AtomicU64 = AtomicU64::new(0);
		let id = ID.fetch_add(1, atomic::Ordering::SeqCst);

		// Create the inner value. This can change if the underlying `IsNode`
		// changes.
		let span = node.span();
		let node = Value::from(node);
		assert!(get_trait!(&node, IsNode).is_some());
		Node {
			id,
			done: Default::default(),
			node: Arc::new(RwLock::new(node)),
			span: Arc::new(RwLock::new(span)),
		}
	}

	pub fn new_at<T: IsNode>(node: T, span: Option<Span>) -> Self {
		let mut node = Self::new(node);
		node.set_span(span);
		node
	}

	/// Globally unique identifier for the node. This does not change with
	/// cloning.
	pub fn id(&self) -> u64 {
		self.id
	}

	pub fn span(&self) -> Option<Span> {
		let span = self.span.read().unwrap();
		span.clone()
	}

	pub fn get<T: IsNode>(&self) -> Option<NodeValueRef<T>> {
		let node = self.node.read().unwrap();
		if node.get::<T>().is_some() {
			Some(NodeValueRef {
				node,
				_phantom: Default::default(),
			})
		} else {
			None
		}
	}

	pub fn get_mut<T: IsNode>(&mut self) -> Option<NodeValueRefMut<T>> {
		let mut node = self.node.write().unwrap();
		if node.get::<T>().is_some() {
			Some(NodeValueRefMut {
				node,
				_phantom: Default::default(),
			})
		} else {
			None
		}
	}

	pub fn set(&mut self, node: Value) {
		assert!(get_trait!(&node, IsNode).is_some());

		let done = self.done.read().unwrap();
		if *done {
			drop(done);
			panic!("cannot set value for a resolved node");
		}

		let mut my_node = self.node.write().unwrap();
		*my_node = node;
	}

	pub fn set_from_node(&mut self, other: Node) {
		let node = { other.node.read().unwrap().clone() };
		let span = other.span();
		self.set(node);
		self.set_span(span);
	}

	pub fn at(mut self, span: Span) -> Self {
		self.set_span(Some(span));
		self
	}

	pub fn is_done(&self) -> bool {
		*self.done.read().unwrap()
	}

	pub fn set_done(&mut self) {
		let mut done = self.done.write().unwrap();
		*done = true;
	}

	pub fn set_span(&mut self, span: Option<Span>) {
		let mut my_span = self.span.write().unwrap();
		*my_span = span;
	}

	pub fn get_span(a: &Node, b: &Node) -> Option<Span> {
		let a = a.span();
		let b = b.span();
		Span::from_range(a, b)
	}

	pub fn val(&self) -> NodeRef {
		let node = self.node.read().unwrap();
		NodeRef { node }
	}

	pub fn val_mut(&mut self) -> NodeRefMut {
		let node = self.node.write().unwrap();
		NodeRefMut { node }
	}
}

/// Possible `eval` results for an [`IsNode`].
pub enum NodeEval {
	/// Node is fully resolved, no more processing is required.
	Complete,

	/// Node evaluated to a new [`IsNode`] value. The value will replace the
	/// current node and continue being evaluated.
	NewValue(Value),

	/// Same as [`NodeEval::NewValue`] but also sets a new position.
	NewValueAndPos(Value, Span),

	/// Similar to [`NodeEval::NewValue`] but uses the content of the given
	/// node.
	FromNode(Node),

	/// The node evaluation depends on the given nodes. The nodes will be fully
	/// resolved before evaluation of the current [`IsNode`] is continued.
	DependsOn(Vec<Node>),
}

//----------------------------------------------------------------------------//
// Trait implementations
//----------------------------------------------------------------------------//

impl<T: IsNode> From<T> for Node {
	fn from(value: T) -> Self {
		Node::new(value)
	}
}

impl HasRepr for Node {
	fn output_repr(&self, output: &mut Repr) -> std::io::Result<()> {
		self.val().output_repr(output)
	}
}

impl Debug for Node {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let node = self.node.read().unwrap();
		write!(f, "{node:?}")
	}
}

impl Display for Node {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let node = self.node.read().unwrap();
		write!(f, "{node}")
	}
}

//----------------------------------------------------------------------------//
// Reference types
//----------------------------------------------------------------------------//

/// Locked read reference to an [`IsNode`].
pub struct NodeRef<'a> {
	node: RwLockReadGuard<'a, Value>,
}

impl<'a> Deref for NodeRef<'a> {
	type Target = dyn IsNode;

	fn deref(&self) -> &Self::Target {
		get_trait!(&*self.node, IsNode).unwrap()
	}
}

/// Locked write reference to a mutable [`IsNode`].
pub struct NodeRefMut<'a> {
	node: RwLockWriteGuard<'a, Value>,
}

impl<'a> Deref for NodeRefMut<'a> {
	type Target = dyn IsNode;

	fn deref(&self) -> &Self::Target {
		get_trait!(&*self.node, IsNode).unwrap()
	}
}

impl<'a> DerefMut for NodeRefMut<'a> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		get_trait_mut!(&mut *self.node, IsNode).unwrap()
	}
}

/// Locked read reference to an [`IsNode`] value.
pub struct NodeValueRef<'a, T: IsNode> {
	node: RwLockReadGuard<'a, Value>,
	_phantom: PhantomData<T>,
}

impl<'a, T: IsNode> Deref for NodeValueRef<'a, T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		self.node.get().unwrap()
	}
}

/// Locked write reference to a mutable [`IsNode`] value.
pub struct NodeValueRefMut<'a, T: IsNode> {
	node: RwLockWriteGuard<'a, Value>,
	_phantom: PhantomData<T>,
}

impl<'a, T: IsNode> Deref for NodeValueRefMut<'a, T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		self.node.get().unwrap()
	}
}

impl<'a, T: IsNode> DerefMut for NodeValueRefMut<'a, T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.node.get_mut().unwrap()
	}
}
