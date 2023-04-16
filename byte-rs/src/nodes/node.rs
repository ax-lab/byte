use std::any::{Any, TypeId};
use std::cell::{Ref, RefCell};
use std::collections::VecDeque;
use std::fmt::Debug;
use std::ops::Deref;
use std::sync::atomic::{self, AtomicU64};
use std::sync::{Arc, Mutex, MutexGuard, RwLock, RwLockReadGuard};

use crate::core::error::*;
use crate::core::input::*;
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
	value: Arc<Mutex<InnerNodeValue>>,
}

/// Root trait implemented for a [`Node`] underlying value.
pub trait IsNode: HasTraits {
	fn eval(&mut self) -> NodeEval;

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
		let node: Box<dyn IsNode> = Box::new(node);
		let node = Arc::new(RwLock::new(node));
		let value = InnerNodeValue {
			node,
			done: false,
			span: None,
		};

		// Wrap everything in a shared mutex.
		let value = Arc::new(Mutex::new(value));
		Node { id, value }
	}

	/// Globally unique identifier for the node. This does not change with
	/// cloning.
	pub fn id(&self) -> u64 {
		self.id
	}

	pub fn span(&self) -> Option<Span> {
		let value = self.value.lock().unwrap();
		value.span.clone()
	}

	pub fn val(&self) -> Arc<RwLock<Box<dyn IsNode>>> {
		let value = self.value.lock().unwrap();
		value.node.clone()
	}

	pub fn get<T: IsNode>(&self) -> Option<NodeRef<T>> {
		let value = self.value.lock().unwrap();
		value.get_value()
	}

	pub fn set(&self, node: Box<dyn IsNode>) {
		let mut value = self.value.lock().unwrap();
		if value.done {
			panic!("cannot set value for a resolved node");
		}

		let node = Arc::new(RwLock::new(node));
		let new_value = InnerNodeValue {
			node,
			done: value.done,
			span: value.span.clone(),
		};
		*value = new_value;
	}

	pub fn at(self, span: Span) -> Self {
		self.set_span(Some(span));
		self
	}

	pub fn is_done(&self) -> bool {
		let value = self.value.lock().unwrap();
		value.done
	}

	pub fn set_done(&self) {
		let mut value = self.value.lock().unwrap();
		value.done = true;
	}

	pub fn set_span(&self, span: Option<Span>) {
		let mut value = self.value.lock().unwrap();
		value.span = span;
	}

	pub fn get_span(a: &Node, b: &Node) -> Option<Span> {
		let sta = a.span();
		let end = b.span();
		let (sta, end) = if sta.is_none() {
			(end, sta)
		} else {
			(sta, end)
		};
		if let Some(sta) = sta {
			let span = if let Some(end) = end {
				let (sta, end) = if sta.sta.offset() < end.sta.offset() {
					(sta, end)
				} else {
					(end, sta)
				};
				Span {
					sta: sta.sta,
					end: end.end,
				}
			} else {
				Span {
					sta: sta.sta.clone(),
					end: sta.sta,
				}
			};
			Some(span)
		} else {
			None
		}
	}
}

/// Possible `eval` results for an [`IsNode`].
pub enum NodeEval {
	/// Node is fully resolved, no more processing is required.
	Complete,

	/// Node evaluated to a new [`IsNode`] value. The value will replace the
	/// current node and continue being evaluated.
	NewValue(Box<dyn IsNode>),

	/// Same as [`NodeEval::NewValue`] but also sets a new position.
	NewValueAndPos(Box<dyn IsNode>, Span),

	/// The node evaluation depends on the given nodes. The nodes will be fully
	/// resolved before evaluation of the current [`IsNode`] is continued.
	DependsOn(Vec<Node>),
}

//----------------------------------------------------------------------------//
// Trait implementations
//----------------------------------------------------------------------------//

impl<T: IsNode> From<T> for Node {
	fn from(value: T) -> Self {
		let span = value.span();
		let node = Node::new(value);
		if let Some(span) = span {
			node.at(span)
		} else {
			node
		}
	}
}

impl Debug for Node {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:?}", self.val())
	}
}

//----------------------------------------------------------------------------//
// Internals
//----------------------------------------------------------------------------//

struct InnerNodeValue {
	done: bool,
	span: Option<Span>,
	node: Arc<RwLock<Box<dyn IsNode>>>,
}

impl InnerNodeValue {
	pub fn get_value<T: IsNode>(&self) -> Option<NodeRef<T>> {
		let guard = self.node.read().unwrap();
		let value = &**guard;
		if value.type_id() == TypeId::of::<T>() {
			let guard = unsafe { std::mem::transmute(guard) };
			let node = NodeRef {
				node: self.node.clone(),
				guard,
			};
			Some(node)
		} else {
			None
		}
	}
}

pub struct NodeRef<T: IsNode> {
	node: Arc<RwLock<Box<dyn IsNode>>>,
	guard: RwLockReadGuard<'static, Box<T>>,
}

impl<T: IsNode> Deref for NodeRef<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		&self.guard
	}
}
