use std::{
	collections::VecDeque,
	fmt::Debug,
	hash::Hash,
	marker::PhantomData,
	mem::ManuallyDrop,
	ptr::NonNull,
	sync::{
		atomic::{AtomicPtr, AtomicU32, AtomicUsize, Ordering},
		RwLock,
	},
};

/*
	Node processing
	===============

	For every node created, use the Expr key to lookup a node func.

	Maintain all nodes in a heap based on the func priority.

	If the node value changes reposition the node in the heap based on the new
	priority.

	Unbound or unresolved nodes are bound to a no-op func with the lowest
	priority.

	When a func is bound to a key, then we need the inverse mapping of key to
	nodes.

	We need a quick way of filtering scopes both ways.

	Maybe list
	----------

	- Nodes should have a done flag to prevent processing discarded nodes

	Implementation details
	======================

	New nodes are allocated in the arena.

	The node key is added to a priority-queue heap and the node itself added to
	that key set.

	When a node key changes, the node is removed from the old key set. The new
	key is added to the priority-queue and the node added to the new key set.

	When a key is bound to a new value, it's relocated in the priority-queue
	based on the new priority. The node set is not changed.

	Bindings for a key are scoped based on the node offset. This means the
	actual key for a node is it's key and its offset.

	A key node set must be such that finding a sub-set of nodes based on their
	offset must be fast. This is used when a key value is set in a particular
	scope.

	## Key lookup

	All defined keys are kept in a hash table and linked to a scope tree.

	## Scope tree

	Each key in the hash table keeps a tree of bindings, where each tree node
	represents that key binding for a specific scope.

	The root node would be the global program scope, with child tree nodes
	representing the key definition in a child scope. Some scopes might be
	unbound, meaning the key is not defined for that scope.

	Child scopes are only created when a key binding is overridden in that
	scope. They do not map 1:1 to actual program scopes.

	Scopes are represented as an offset range. Offsets are unique across the
	whole program.

	A node in the scope tree can be split when that key is redefined for a
	child scope. The associated nodes will be split along side.

	## Key bindings & Priority Queue

	A key binding defines the value of a key for a specific scope. The binding
	defines an operator to be applied to the nodes in that scope with a given
	precedence.

	For the actual implementation, each scope tree node is associated with a
	precedence and added to the priority-queue heap. The queue will process
	all scopes that have the next highest priority as a single transactional
	step.

	If a key is rebound, its scope tree node priorities will be reevaluated
	alongside their position in the heap.

	## Priority re-queue

	Nodes that are processed should be removed from the queue. When nodes
	are created, they may be added to a scope tree node that was already
	processed previously, in which case that node would get re-added to the
	queue, with only the new nodes.

	## Node List

	The node list is the actual value of a scope tree node. Nodes here are
	sorted by their offset to allow fast positional lookup and splitting.

	The node list should be efficient enough to handle all nodes in the
	program, if need be. It should also allow fast insertion and removal.

	## Node updates

	Nodes are updated when their expression updates.

	When processing node operators, the changes are queued but not applied
	immediately. The entire set of changes must be validated for conflicts
	before applying.

	Once changes are validated, their application will generate new values
	for the existing nodes, as well as node changes. Those will then be
	applied for the above structure.

	Node operators can also set new bindings, which will also re-evaluate
	the scope tree and key bindings.

	## Node structure

	The node hierarchical structure is separate from the scope and key bindings
	representation. When applying operators to nodes, they are applied in their
	parent-child context.

	Operators should probably take the scope into consideration to limit their
	range of application, where applicable.

*/

pub mod arena;
use arena::*;

pub mod node;
pub use node::*;

/// Marker types that define a [`Node`] and its associated types.
pub trait IsNode: Copy + 'static {
	/// Value for the node.
	type Expr<'a>: IsExpr<'a, Self>;

	/// Key based on [`IsNode::Expr`] used to lookup node data and operations
	/// in the scope.
	type Key: Default + Clone + Hash + Eq + PartialEq + 'static;
	type Val: Default + Clone + Hash + Eq + PartialEq + 'static;

	type Precedence: Default + Ord + PartialOrd + 'static;

	fn get_precedence(val: &Self::Val) -> Self::Precedence {
		let _ = val;
		<Self::Precedence as Default>::default()
	}

	fn apply(val: &Self::Val, node: Node<Self>) {
		let _ = (val, node);
	}
}

/// Types that are used as values for a [`Node`].
pub trait IsExpr<'a, T: IsNode + 'a>: 'a + Debug + Send + Sync {
	fn key(&self) -> T::Key {
		<T::Key as Default>::default()
	}

	fn offset(&self) -> usize {
		0
	}

	fn children(&self) -> NodeIterator<'a, T> {
		NodeIterator::empty()
	}
}

//====================================================================================================================//
// Store
//====================================================================================================================//

/// Provides storage for [`NodeSet`] and its set of [`Node`]. The store owns all
/// set and node data.
pub struct NodeStore<T: IsNode> {
	buffer: Buffer,
	nodes: RawArena,
	_node: PhantomData<T>,
}

impl<T: IsNode> NodeStore<T> {
	pub fn new<'a>() -> Self {
		Self {
			buffer: Buffer::default(),
			nodes: RawArena::for_type::<NodeData<T>>(1024),
			_node: Default::default(),
		}
	}

	/// Create a new [`NodeSet`] backed by this store.
	pub fn new_node_set(&self) -> NodeSet<T> {
		NodeSet::new(self)
	}
}

impl<T: IsNode> Default for NodeStore<T> {
	fn default() -> Self {
		Self::new()
	}
}

//====================================================================================================================//
// Tests
//====================================================================================================================//

#[cfg(test)]
mod tests {
	use std::sync::Arc;

	use super::*;

	#[test]
	fn test_simple() {
		let store = NodeStore::<Test>::new();
		let mut set = store.new_node_set();
		let list = make_simple_list(&mut set);
		let actual = format!("{list:?}");
		assert_eq!(actual, "List([ Zero, Node(Zero) ])");
	}

	#[test]
	fn test_compiler() {
		let data = NodeStore::<Test>::new();
		let mut compiler = make_compiler(&data);
		let node1_a = compiler.add_num(1);
		let node2_a = compiler.add_num(2);

		let zero = compiler.get(0);
		let node1_b = compiler.get(1);
		let node2_b = compiler.get(2);

		assert_eq!(node1_a, node1_b);
		assert_eq!(node2_a, node2_b);
		assert_eq!(format!("{zero:?}"), "Zero");
		assert_eq!(format!("{node1_a:?}"), "Number(1)");
		assert_eq!(format!("{node2_a:?}"), "Number(2)");
		drop(data);
	}

	#[test]
	fn test_expr_drops() {
		let store = NodeStore::<DropTest>::new();
		let mut set = store.new_node_set();
		let num = 100;

		let counter: Arc<RwLock<usize>> = Default::default();
		for _ in 0..num {
			set.new_node(DropExpr::new(counter.clone()));
		}

		assert_eq!(*counter.read().unwrap(), num);
		drop(store);
		assert_eq!(*counter.read().unwrap(), 0);
	}

	//----------------------------------------------------------------------------------------------------------------//
	// Basic test fixture
	//----------------------------------------------------------------------------------------------------------------//

	#[derive(Copy, Clone)]
	struct Test;

	#[derive(Copy, Clone, Debug)]
	enum TestExpr<'a> {
		Zero,
		Node(Node<'a, Test>),
		List(NodeList<'a, Test>),
		Number(i32),
	}

	impl IsNode for Test {
		type Expr<'a> = TestExpr<'a>;
		type Key = ();
		type Val = ();
		type Precedence = ();
	}

	impl<'a> IsExpr<'a, Test> for TestExpr<'a> {
		fn children(&self) -> NodeIterator<'a, Test> {
			match self {
				TestExpr::Zero => NodeIterator::empty(),
				TestExpr::Node(node) => NodeIterator::single(node),
				TestExpr::List(list) => list.into_iter(),
				TestExpr::Number(..) => NodeIterator::empty(),
			}
		}
	}

	fn make_simple_list<'a>(store: &mut NodeSet<'a, Test>) -> Node<'a, Test> {
		let zero = store.new_node(TestExpr::Zero);
		let node = store.new_node(TestExpr::Node(zero));
		let list = store.new_node(TestExpr::List(NodeList::pair(zero, node)));
		list
	}

	//----------------------------------------------------------------------------------------------------------------//
	// Compiler simulation fixture
	//----------------------------------------------------------------------------------------------------------------//

	// This is to simulate the node usage within the compiler environment and
	// make sure lifetimes and borrows work as expected.

	fn make_compiler<'a>(data: &'a NodeStore<Test>) -> Compiler<'a> {
		let mut compiler = Compiler::new(data);
		compiler.add_zero();
		compiler
	}

	struct Compiler<'a> {
		set: NodeSet<'a, Test>,
		nodes: Vec<Node<'a, Test>>,
	}

	impl<'a> Compiler<'a> {
		pub fn new(store: &'a NodeStore<Test>) -> Self {
			Self {
				set: store.new_node_set(),
				nodes: Default::default(),
			}
		}

		pub fn add_zero(&mut self) -> Node<'a, Test> {
			let node = self.set.new_node(TestExpr::Zero);
			self.nodes.push(node);
			node
		}

		pub fn add_num(&mut self, value: i32) -> Node<'a, Test> {
			let node = self.set.new_node(TestExpr::Number(value));
			self.nodes.push(node);
			node
		}

		pub fn get(&self, index: usize) -> Node<'a, Test> {
			self.nodes[index]
		}
	}

	//----------------------------------------------------------------------------------------------------------------//
	// Drop test fixture
	//----------------------------------------------------------------------------------------------------------------//

	#[derive(Copy, Clone)]
	struct DropTest;

	impl IsNode for DropTest {
		type Expr<'a> = DropExpr;
		type Key = ();
		type Val = ();
		type Precedence = ();
	}

	#[derive(Debug)]
	struct DropExpr(Arc<RwLock<usize>>);

	impl DropExpr {
		pub fn new(value: Arc<RwLock<usize>>) -> Self {
			{
				let mut value = value.write().unwrap();
				*value += 1;
			}
			Self(value)
		}
	}

	impl Drop for DropExpr {
		fn drop(&mut self) {
			let mut value = self.0.write().unwrap();
			*value -= 1;
		}
	}

	impl<'a> IsExpr<'a, DropTest> for DropExpr {}

	//----------------------------------------------------------------------------------------------------------------//
	// Type assertions
	//----------------------------------------------------------------------------------------------------------------//

	// Assert individual node types implement the intended auto-traits.
	const _: () = {
		fn assert_safe<T: Send + Sync>() {}
		fn assert_copy<T: Copy>() {}

		fn assert_all() {
			assert_safe::<NodeData<Test>>();
			assert_safe::<Node<Test>>();
			assert_safe::<TestExpr>();

			assert_copy::<Node<Test>>();
			assert_copy::<TestExpr>();
		}
	};
}
