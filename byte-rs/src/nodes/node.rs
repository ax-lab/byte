use std::fmt::{Debug, Display};
use std::io::Write;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{self, AtomicU64};
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::core::repr::*;
use crate::core::*;

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
	scope: Scope,
	node: Arc<RwLock<Value>>,
	span: Arc<RwLock<Option<Span>>>,
	done: Arc<RwLock<bool>>,
}

impl PartialEq for Node {
	fn eq(&self, other: &Self) -> bool {
		self.id == other.id
	}
}

impl Node {
	pub fn new<T: IsNode>(node: T, scope: Scope) -> Self {
		// Generate a unique ID for each instance. The ID will remain constant
		// even if the underlying value changes and is preserved by cloning.
		static ID: AtomicU64 = AtomicU64::new(0);
		let id = ID.fetch_add(1, atomic::Ordering::SeqCst);

		// Create the inner value. This can change if the underlying `IsNode`
		// changes.
		let node = Value::from(node);
		assert!(get_trait!(&node, IsNode).is_some());
		Node {
			id,
			scope,
			done: Default::default(),
			node: Arc::new(RwLock::new(node)),
			span: Arc::new(RwLock::new(None)),
		}
	}

	pub fn new_at<T: IsNode>(node: T, scope: Scope, span: Option<Span>) -> Self {
		let mut node = Self::new(node, scope);
		if span.is_some() {
			node.set_span(span);
		}
		node
	}

	pub fn scope(&self) -> Scope {
		self.scope.clone()
	}

	/// Globally unique identifier for the node. This does not change with
	/// cloning.
	pub fn id(&self) -> u64 {
		self.id
	}

	pub fn span_from(a: &Node, b: &Node) -> Option<Span> {
		Span::from_range(a.span(), b.span())
	}

	pub fn span_from_list(items: &Vec<Node>) -> Option<Span> {
		let first = items.first();
		let last = items.last();
		let first = first.and_then(|x| x.span());
		let last = last.and_then(|x| x.span());
		Span::from_range(first, last)
	}

	pub fn span(&self) -> Option<Span> {
		let span = {
			let span = self.span.read().unwrap();
			span.clone()
		};
		span.or_else(|| self.val().span())
	}

	#[allow(unused)]
	pub fn is<T: IsNode>(&self) -> bool {
		self.get::<T>().is_some()
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

	#[allow(unused)]
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

	pub fn repr_for_msg(&self) -> String {
		let mut repr = Repr::new(ReprMode::Debug, ReprFormat::Minimal);
		let _ = self.output_repr(&mut repr);
		let repr = repr.to_string();
		if repr.contains('\n') {
			let lines: Vec<_> = repr.lines().map(|x| format!("    {x}")).collect();
			format!("\n\n{}\n", lines.join("\n"))
		} else {
			repr
		}
	}
}

/// Possible `eval` results for an [`IsNode`].
#[allow(unused)]
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

impl NodeEval {
	pub fn depends_on(input: &Vec<Node>) -> NodeEval {
		let pending: Vec<_> = input.iter().filter(|x| !x.is_done()).cloned().collect();
		if pending.len() > 0 {
			NodeEval::DependsOn(pending)
		} else {
			NodeEval::Complete
		}
	}

	pub fn check(&mut self, node: &Node) {
		if !node.is_done() {
			match self {
				NodeEval::Complete => {
					*self = NodeEval::DependsOn(vec![node.clone()]);
				}
				NodeEval::DependsOn(vec) => {
					vec.push(node.clone());
				}
				_ => panic!("NodeEval::check: invalid value"),
			}
		}
	}
}

//----------------------------------------------------------------------------//
// Trait implementations
//----------------------------------------------------------------------------//

impl HasRepr for Node {
	fn output_repr(&self, output: &mut Repr) -> std::io::Result<()> {
		self.val().output_repr(output)?;
		if output.is_debug() && !output.is_compact() {
			if let Some(span) = self.span() {
				write!(output, " @")?;
				span.output_repr(&mut output.minimal().display())?;
			}
		}
		Ok(())
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
