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

	Data structures and details
	---------------------------

	- Node needs its heap index (to reposition when the key changes)
	  - If enough nodes change, it might be cheaper to rebuild the heap?
	- A hash table of node key to possible node func, and then a scope filter
	- Nodes should have a done flag to prevent processing discarded nodes

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
	type Key: Clone + Hash + 'static;
}

/// Types that are used as values for a [`Node`].
pub trait IsExpr<'a, T: IsNode + 'a>: 'a + Debug + Send + Sync {
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
		let set = store.new_node_set();
		let list = make_simple_list(&set);

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
		let set = store.new_node_set();
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

	fn make_simple_list<'a>(store: &'a NodeSet<'a, Test>) -> Node<'a, Test> {
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
