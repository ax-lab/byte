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
	node: Arc<RwLock<Value>>,
	span: Arc<RwLock<Option<Span>>>,
	done: Arc<RwLock<bool>>,
	link: Arc<RwLock<NodeLink>>,
	errors: Arc<RwLock<ErrorList>>,
}

impl Default for Node {
	fn default() -> Self {
		Node {
			id: 0,
			node: Arc::new(RwLock::new(Value::from(NoneValue))),
			span: Default::default(),
			done: Arc::new(RwLock::new(true)),
			link: Default::default(),
			errors: Default::default(),
		}
	}
}

struct NodeList {
	parent: Option<Node>,
	head: Node,
	last: Node,
}

#[derive(Default)]
struct NodeLink {
	list: Option<Arc<RwLock<NodeList>>>,
	prev: Option<Node>,
	next: Option<Node>,
}

impl NodeLink {
	pub fn as_list(&mut self, node: &Node) -> Arc<RwLock<NodeList>> {
		let list = self.list.get_or_insert_with(|| {
			Arc::new(RwLock::new(NodeList {
				parent: None,
				head: node.clone(),
				last: node.clone(),
			}))
		});
		list.clone()
	}
}

impl PartialEq for Node {
	fn eq(&self, other: &Self) -> bool {
		self.id == other.id
	}
}

#[allow(unused)]
impl Node {
	pub fn new_root() -> Self {
		Self::new_detached(RootValue)
	}

	pub fn new<T: IsNode>(value: T) -> Self {
		Self::new_detached(value)
	}

	pub fn new_detached<T: IsNode>(value: T) -> Self {
		// Generate a unique ID for each instance. The ID will remain constant
		// even if the underlying value changes and is preserved by cloning.
		static ID: AtomicU64 = AtomicU64::new(1);
		let id = ID.fetch_add(1, atomic::Ordering::SeqCst);

		// Create the inner value. This can change if the underlying `IsNode`
		// changes.
		let node = Value::from(value);
		assert!(get_trait!(&node, IsNode).is_some());
		let node = Node {
			id,
			done: Default::default(),
			node: Arc::new(RwLock::new(node)),
			span: Arc::new(RwLock::new(None)),
			errors: Default::default(),
			link: Default::default(),
		};
		node
	}

	/// Globally unique identifier for the node. This does not change with
	/// cloning.
	pub fn id(&self) -> u64 {
		self.id
	}

	//================================================================================================================//
	// Hierarchy
	//================================================================================================================//

	pub fn is_detached(&self) -> bool {
		if let Some(parent) = self.parent() {
			parent.is_detached()
		} else {
			!self.is::<RootValue>()
		}
	}

	pub fn parent(&self) -> Option<Node> {
		self.get_list()
			.and_then(|x| x.read().unwrap().parent.clone())
	}

	pub fn up(&self) -> Node {
		self.parent().unwrap_or_else(|| self.clone())
	}

	pub fn prev(&self) -> Option<Node> {
		self.link.read().unwrap().prev.clone()
	}

	pub fn next(&self) -> Option<Node> {
		self.link.read().unwrap().next.clone()
	}

	pub fn node_at(&self, mut n: usize) -> Option<Node> {
		let mut curr = self.clone();
		for _ in 0..n {
			if let Some(node) = curr.next() {
				curr = node;
			} else {
				return None;
			}
		}
		Some(curr)
	}

	pub fn head(&self) -> Node {
		self.get_list()
			.map(|x| x.read().unwrap().head.clone())
			.unwrap_or_else(|| self.clone())
	}

	pub fn last(&self) -> Node {
		self.get_list()
			.map(|x| x.read().unwrap().last.clone())
			.unwrap_or_else(|| self.clone())
	}

	fn set_parent(&mut self, parent: &Node) {
		let list = {
			let mut link = self.link.write().unwrap();
			link.as_list(self)
		};
		let mut list = list.write().unwrap();
		list.parent = Some(parent.clone());
	}

	pub fn extract(&mut self) {
		let mut curr = self.link.write().unwrap();
		let mut list = curr.as_list(self);
		let mut list = list.write().unwrap();

		// update previous link
		if let Some(ref prev) = curr.prev {
			let mut prev = prev.link.write().unwrap();
			prev.next = curr.next.clone();
		} else if let Some(ref next) = curr.next {
			list.head = next.clone();
		}

		// update next link
		if let Some(ref next) = curr.next {
			let mut next = next.link.write().unwrap();
			next.prev = curr.prev.clone();
		} else if let Some(ref prev) = curr.prev {
			list.last = prev.clone();
		}

		drop(list);
		curr.prev = None;
		curr.next = None;
		curr.list = None;
	}

	pub fn split_next(&mut self) -> Option<Node> {
		let mut curr = self.link.write().unwrap();
		let mut list = curr.as_list(self);
		let mut list = list.write().unwrap();
		if let Some(ref next) = curr.next {
			let mut next = next.clone();
			let mut next_link = next.link.write().unwrap();
			next_link.list = Some(Arc::new(RwLock::new(NodeList {
				parent: list.parent.clone(),
				head: next.clone(),
				last: list.last.clone(),
			})));

			let new_list = next_link.list.clone();
			let mut iter = next_link.next.clone();
			while let Some(node) = iter {
				let mut link = node.link.write().unwrap();
				link.list = new_list.clone();
				iter = link.next.clone();
			}

			list.last = self.clone();
			curr.next = None;

			drop(next_link);
			Some(next)
		} else {
			None
		}
	}

	pub fn push_value<T: IsNode>(&mut self, value: T) {
		let node = Self::new_detached(value);
		self.push(node);
	}

	pub fn push(&mut self, node: Node) {
		self.last().append(node)
	}

	pub fn append(&mut self, node: Node) {
		self.do_insert(node, true, None)
	}

	pub fn insert(&mut self, node: Node) {
		self.do_insert(node, false, None)
	}

	pub fn replace(&mut self, node: Node, count: usize) {
		self.do_insert(node, false, Some(count))
	}

	fn do_insert(&mut self, node: Node, append_only: bool, replace: Option<usize>) {
		let result = loop {
			if self.id() == node.id() {
				break Err("cannot append a node to itself");
			}

			// Lock the underlying list of both nodes. We use the node ID as
			// the global order to acquire the locks to avoid a deadlock:

			// First lock the nodes themselves
			let (mut self_link, mut node_link) = if self.id() < node.id() {
				let self_link = self.link.write().unwrap();
				let node_link = node.link.write().unwrap();
				(self_link, node_link)
			} else {
				let node_link = node.link.write().unwrap();
				let self_link = self.link.write().unwrap();
				(self_link, node_link)
			};

			// we must be the last node of our list
			if append_only && self_link.next.is_some() {
				break Err("can only append to the last node in a list");
			}

			// the given node must be the head of its list
			if node_link.prev.is_some() {
				break Err("cannot append a node that is part of another list");
			}

			// make sure the nodes are initialized as lists
			let self_list = self_link.as_list(&self);
			let node_list = node_link.as_list(&node);

			// The lists are shared between all nodes and could still be
			// acquired through other nodes. To avoid a deadlock we try to
			// lock the lists, and if we fail we release the locks and try
			// again.
			let (mut self_list, mut node_list) = if self.id() < node.id() {
				let self_list = if let Ok(value) = self_list.try_write() {
					value
				} else {
					continue;
				};
				let node_list = if let Ok(value) = node_list.try_write() {
					value
				} else {
					continue;
				};
				(self_list, node_list)
			} else {
				let node_list = if let Ok(value) = node_list.try_write() {
					value
				} else {
					continue;
				};
				let self_list = if let Ok(value) = self_list.try_write() {
					value
				} else {
					continue;
				};
				(self_list, node_list)
			};

			// lists must not be the same
			if self_list.head.id() == node_list.head.id() {
				break Err("cannot append a node to its own list");
			}

			// if the other node has a parent, then it must be the same parent
			if node_list.parent.is_some() && node_list.parent != self_list.parent {
				break Err("cannot append a node from a different parent");
			}

			let (node, mut node_link) = if let Some(replace) = replace {
				if replace == 0 {
					let mut new_node = Self::new_detached(NoneValue);
					new_node.set_value_from_node(&self);

					let mut new_link = new_node.link.write().unwrap();
					new_link.list = self_link.list.clone();
					new_link.prev = Some(self.clone());
					new_link.next = self_link.next.clone();
					if let Some(next) = new_link.next.as_ref() {
						let mut next = next.link.write().unwrap();
						next.prev = Some(new_node.clone());
					}
					drop(new_link);

					self_link.next = Some(new_node.clone());
					if self_list.last.id() == self.id() {
						self_list.last = new_node.clone();
					}
				}

				if replace > 1 {
					// remove extra nodes being replaced from the list
					let mut after = self_link.next.clone();
					for _ in 1..replace {
						if let Some(next) = after {
							let mut link = next.link.write().unwrap();
							after = link.next.take();
							self_link.next = after.clone();
							if let Some(ref after) = after {
								let mut link = after.link.write().unwrap();
								link.prev = Some(self.clone());
							} else {
								self_list.last = self.clone();
							}
							link.prev = None;
							link.list = None;
						} else {
							break;
						}
					}
				}

				// replace the current value by the given's node
				self.clone().set_value_from_node(&node);

				// we effectively "inserted" the node head, so we move to the
				// next one
				if let Some(next) = node_link.next.clone() {
					node_link.next = None;
					node_link.list = None;
					(next, None)
				} else {
					// if there is no next one, then we are done
					break Ok(());
				}
			} else {
				(node.clone(), Some(node_link))
			};

			let mut node_link = node_link.unwrap_or_else(|| node.link.write().unwrap());

			// update the reference to the last node
			if let Some(next) = self_link.next.as_ref() {
				// we need to update the last node of the appended list to
				// point to our next node
				if node_list.last.id() == node.id() {
					node_link.next = self_link.next.clone();
				} else {
					let mut last_link = node_list.last.link.write().unwrap();
					last_link.next = self_link.next.clone();
				}

				let mut next = next.link.write().unwrap();
				next.prev = Some(node_list.last.clone());
			} else {
				self_list.last = node_list.last.clone();
			}

			// now we can update our next node to point to the appended list
			self_link.next = Some(node.clone());
			node_link.prev = Some(self.clone());

			// finally update every item in the node list to point to our own
			// list
			node_link.list = self_link.list.clone();

			let mut current = node_link.next.clone();
			while let Some(node) = current {
				let mut link = node.link.write().unwrap();
				link.list = self_link.list.clone();
				current = link.next.clone();
			}

			break Ok(());
		};

		result.unwrap();
	}

	fn get_list(&self) -> Option<Arc<RwLock<NodeList>>> {
		let link = self.link.read().unwrap();
		link.list.clone()
	}

	//================================================================================================================//
	// Span
	//================================================================================================================//

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

	pub fn at(mut self, span: Option<Span>) -> Self {
		if span.is_some() {
			self.set_span(span);
		}
		self
	}

	pub fn span(&self) -> Option<Span> {
		self.span.read().unwrap().clone()
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

	//================================================================================================================//
	// Error handling
	//================================================================================================================//

	pub fn errors_mut(&mut self) -> NodeErrorListMut {
		NodeErrorListMut {
			errors: self.errors.write().unwrap(),
		}
	}

	pub fn has_errors(&self) -> bool {
		let errors = self.errors.read().unwrap();
		!errors.empty()
	}

	pub fn errors(&self) -> ErrorList {
		self.errors.read().unwrap().clone()
	}

	//================================================================================================================//
	// Value
	//================================================================================================================//

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

	pub fn set_value_from_node(&mut self, other: &Node) {
		let (node, span, done, errors) = {
			(
				other.node.read().unwrap().clone(),
				other.span(),
				other.is_done(),
				other.errors(),
			)
		};
		*self.node.write().unwrap() = node;
		*self.span.write().unwrap() = span;
		*self.done.write().unwrap() = done;
		*self.errors.write().unwrap() = errors;
	}

	pub fn is_done(&self) -> bool {
		*self.done.read().unwrap()
	}

	pub fn set_done(&mut self) {
		let mut done = self.done.write().unwrap();
		*done = true;
	}

	pub fn val(&self) -> NodeRef {
		let value = self.node.read().unwrap().clone();
		NodeRef { value }
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

	/// Node has changed and needs to be reevaluated.
	Changed,

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

impl Node {
	fn output_own_repr(&self, output: &mut Repr) -> std::io::Result<()> {
		self.val().output_repr(output)?;
		if output.is_debug() && !output.is_compact() {
			if let Some(span) = self.span() {
				write!(output, " @")?;
				span.output_repr(&mut output.minimal().display())?;
			}
		}
		Ok(())
	}

	pub fn output_list(&self, output: &mut Repr) -> std::io::Result<()> {
		if output.is_compact() {
			write!(output, "[ ")?;
			self.output_own_repr(output)?;
			let mut next = self.next();
			while let Some(node) = next {
				write!(output, ", ")?;
				node.output_own_repr(output)?;
				next = node.next();
			}
			write!(output, " ]")?;
		} else {
			{
				let mut output = output.indented();
				write!(output, "[\n ")?;
				self.output_own_repr(&mut output)?;
				let mut next = self.next();
				while let Some(node) = next {
					write!(output, "\n")?;
					node.output_own_repr(&mut output)?;
					next = node.next();
				}
			}
		}
		Ok(())
	}
}

impl HasRepr for Node {
	fn output_repr(&self, output: &mut Repr) -> std::io::Result<()> {
		if let Some(next) = self.next() {
			if output.is_compact() {
				write!(output, "[ ")?;
				self.output_own_repr(output)?;
				write!(output, ", ")?;
				next.output_own_repr(output)?;
				let mut next = next.next();
				while let Some(node) = next {
					write!(output, ", ")?;
					node.output_own_repr(output)?;
					next = node.next()
				}
				write!(output, " ]")?;
			} else {
				{
					let mut output = output.indented();
					write!(output, "[\n")?;
					self.output_own_repr(&mut output)?;
					write!(output, "\n")?;
					next.output_own_repr(&mut output)?;
					let mut next = next.next();
					while let Some(node) = next {
						write!(output, "\n")?;
						node.output_own_repr(&mut output)?;
						next = node.next()
					}
				}
				write!(output, "\n]")?;
			}
			Ok(())
		} else {
			self.output_own_repr(output)
		}
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
		let mut repr = Repr::new(ReprMode::Display, ReprFormat::Minimal);
		let _ = self.output_repr(&mut repr);
		write!(f, "{repr}")
	}
}

//--------------------------------------------------------------------------------------------------------------------//
// Reference types
//--------------------------------------------------------------------------------------------------------------------//

/// Reference to the [`IsNode`] value of a [`Node`].
pub struct NodeRef {
	value: Value,
}

impl Deref for NodeRef {
	type Target = dyn IsNode;

	fn deref(&self) -> &Self::Target {
		get_trait!(&self.value, IsNode).unwrap()
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

/// Locked write reference to an [`ErrorList`].
pub struct NodeErrorListMut<'a> {
	errors: RwLockWriteGuard<'a, ErrorList>,
}

impl<'a> Deref for NodeErrorListMut<'a> {
	type Target = ErrorList;

	fn deref(&self) -> &Self::Target {
		&self.errors
	}
}

impl<'a> DerefMut for NodeErrorListMut<'a> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.errors
	}
}

//====================================================================================================================//
// Special values
//====================================================================================================================//

#[derive(Clone)]
struct RootValue;

has_traits!(RootValue: IsNode, HasRepr);

impl IsNode for RootValue {
	fn eval(&self, _node: Node) -> NodeEval {
		NodeEval::Complete
	}
}

impl HasRepr for RootValue {
	fn output_repr(&self, output: &mut Repr) -> std::io::Result<()> {
		write!(output, "Root")
	}
}

#[derive(Clone)]
struct NoneValue;

has_traits!(NoneValue: IsNode, HasRepr);

impl IsNode for NoneValue {
	fn eval(&self, _node: Node) -> NodeEval {
		NodeEval::Complete
	}
}

impl HasRepr for NoneValue {
	fn output_repr(&self, output: &mut Repr) -> std::io::Result<()> {
		write!(output, "None")
	}
}

//====================================================================================================================//
// Tests
//====================================================================================================================//

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	pub fn test_simple_list_detached() {
		let mut node = Node::new_detached(Test(1));
		node.push_value(Test(2));
		node.push_value(Test(3));
		check_list(&node, "[ 1, 2, 3 ]");
		assert!(node.is_detached());
		assert!(node.parent().is_none());
		check(node.last(), 3);
		check(node.last().head(), 1);
		check_opt(node.next(), 2);
		check_opt(node.next().and_then(|x| x.next()), 3);
	}

	#[test]
	pub fn test_append_lists() {
		let root = Node::new_root();
		let mut node = Node::new_detached(Test(1));
		node.set_parent(&root);
		node.push_value(Test(2));
		node.push_value(Test(3));
		assert!(!node.is_detached());

		let mut list = Node::new_detached(Test(4));
		list.push_value(Test(5));
		list.push_value(Test(6));

		node.push(list.clone());
		assert!(!list.is_detached());

		check_list(&node, "[ 1, 2, 3, 4, 5, 6 ]");
		check_list_p(&list, "[ 4, 5, 6 ]");

		assert!(node.parent() == list.parent());
		check(node.last(), 6);
		check(node.last().head(), 1);
		check(list.last(), 6);
		check(list.last().head(), 1);

		check_opt(list.prev(), 3);
	}

	#[test]
	pub fn test_insert() {
		let mut node = Node::new_detached(Test(1));
		node.push_value(Test(2));
		node.push_value(Test(7));

		let list = Node::new_detached(Test(3));
		node.next().unwrap().insert(list);

		check_list(&node, "[ 1, 2, 3, 7 ]");
		check(node.last(), 7);
		check_opt(node.last().prev(), 3);

		let mut list = Node::new_detached(Test(4));
		list.push_value(Test(5));
		list.push_value(Test(6));
		node.last().prev().unwrap().insert(list.clone());

		check_list(&node, "[ 1, 2, 3, 4, 5, 6, 7 ]");

		check_opt(list.prev(), 3);
		check_opt(list.prev().unwrap().next(), 4);
	}

	#[test]
	pub fn test_replace() {
		let root = Node::new_root();
		let mut node = Node::new_detached(Test(1));
		node.set_parent(&root);

		node.push_value(Test(2));
		node.push_value(Test(3));

		node.replace(Node::new_detached(Test(0)), 0);
		check(node.clone(), 0);
		check_list(&node, "[ 0, 1, 2, 3 ]");

		let mut list = Node::new_detached(Test(-3));
		list.push_value(Test(-2));
		list.push_value(Test(-1));

		node.replace(list, 0);
		check(node.clone(), -3);
		check_list(&node, "[ -3, -2, -1, 0, 1, 2, 3 ]");

		let next = node.next().unwrap();
		node.next()
			.unwrap()
			.replace(Node::new_detached(Test(42)), 2);
		check_list(&node, "[ -3, 42, 0, 1, 2, 3 ]");
		check(next, 42);

		node.last()
			.prev()
			.unwrap()
			.replace(Node::new_detached(Test(30)), 99);
		check_list(&node, "[ -3, 42, 0, 1, 30 ]");

		let mut list = Node::new_detached(Test(10));
		list.push_value(Test(20));
		node.replace(list, 4);
		check_list(&node, "[ 10, 20, 30 ]");
		check(node.head(), 10);

		node.replace(Node::new_detached(Test(0)), 3);
		check_list(&node, "0");
		check(node.head(), 0);
		check(node.last(), 0);

		assert!(!node.is_detached());
		assert!(node.parent() == Some(root));
	}

	#[test]
	fn test_extract() {
		let root = Node::new_root();
		let mut n1 = Node::new_detached(Test(1));
		n1.set_parent(&root);

		n1.push_value(Test(2));
		n1.push_value(Test(3));
		n1.push_value(Test(4));
		n1.push_value(Test(5));

		let mut n3 = n1.node_at(2).unwrap();
		n3.extract();

		check_list(&n1, "[ 1, 2, 4, 5 ]");
		check_list(&n3, "3");
		assert!(n3.is_detached());
		assert!(n3.parent().is_none());
		assert!(n1.parent() == Some(root.clone()));

		let n2 = n1.node_at(1).unwrap();
		n1.extract();

		check_list(&n2, "[ 2, 4, 5 ]");
		check_list(&n1, "1");
		assert!(n1.is_detached());
		assert!(n1.parent().is_none());
		assert!(n2.parent() == Some(root.clone()));
		assert!(n2.head() == n2);

		let mut n5 = n2.node_at(2).unwrap();
		n5.extract();

		check_list(&n2, "[ 2, 4 ]");
		check_list(&n5, "5");
		assert!(n1.is_detached());
		assert!(n1.parent().is_none());
		assert!(n2.parent() == Some(root.clone()));
	}

	#[test]
	fn test_split() {
		let root = Node::new_root();
		let mut n1 = Node::new_detached(Test(1));
		n1.set_parent(&root);

		n1.push_value(Test(2));
		n1.push_value(Test(3));
		n1.push_value(Test(4));
		n1.push_value(Test(5));

		let mut n3 = n1.node_at(2).unwrap();

		let n4 = n3.split_next().unwrap();

		assert!(n1.last().split_next().is_none());
		assert!(n4.last().split_next().is_none());

		check_list(&n1, "[ 1, 2, 3 ]");
		check_list(&n4, "[ 4, 5 ]");

		check_list_p(&n3, "3");
		assert!(n3.head() == n1);

		assert!(n1.parent().as_ref() == Some(&root));
		assert!(n4.parent().as_ref() == Some(&root));

		let n2 = n1.split_next().unwrap();
		check_list(&n1, "1");
		check_list(&n2, "[ 2, 3 ]");

		assert!(n1.parent().as_ref() == Some(&root));
		assert!(n2.parent().as_ref() == Some(&root));
		assert!(n2.last() == n3);
		assert!(n3.head() == n2);
	}

	//================================================================================================================//
	// Helpers
	//================================================================================================================//

	fn check_list(node: &Node, repr: &str) {
		do_check_list(node, repr, false)
	}

	fn check_list_p(node: &Node, repr: &str) {
		do_check_list(node, repr, true)
	}

	fn do_check_list(node: &Node, repr: &str, partial: bool) {
		let node = node.clone();
		assert_eq!(node.to_string(), repr);

		let mut prev = node.clone();
		let mut next = node.next();
		while let Some(curr) = next.clone() {
			if partial {
				assert!(
					curr.head() == node.head(),
					"invalid head in node `{curr}`, after `{prev}`"
				);
			} else {
				assert!(
					curr.head() == node,
					"invalid head in node `{curr}`, after `{prev}`"
				);
			}
			assert!(
				curr.last() == node.last(),
				"invalid last in node `{curr}`, after `{prev}`"
			);
			assert!(
				curr.parent() == node.parent(),
				"invalid parent in node `{curr}`, after `{prev}`"
			);
			assert!(
				curr.prev() == Some(prev.clone()),
				"invalid prev in node `{curr}`, after `{prev}` (is `{:?}`)",
				curr.prev(),
			);
			next = curr.next();
			prev = curr.clone();
		}

		assert!(prev == node.last(), "prev is not last node: {prev}");
	}

	fn check(node: Node, value: i32) {
		let actual = node.get::<Test>().unwrap().clone();
		assert_eq!(actual, Test(value));
	}

	fn check_opt(node: Option<Node>, value: i32) {
		let actual = node.unwrap().get::<Test>().unwrap().clone();
		assert_eq!(actual, Test(value));
	}

	#[derive(Clone, Eq, PartialEq, Debug)]
	struct Test(i32);

	has_traits!(Test: IsNode, HasRepr);

	impl IsNode for Test {
		fn eval(&self, _node: Node) -> NodeEval {
			NodeEval::Complete
		}
	}

	impl HasRepr for Test {
		fn output_repr(&self, output: &mut Repr) -> std::io::Result<()> {
			write!(output, "{}", self.0)
		}
	}
}
