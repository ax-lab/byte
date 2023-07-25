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

/// Marker types that define a [`Node`] and its associated types.
pub trait IsNode: Copy + 'static {
	type Expr<'a>: IsExpr<'a, Self>;

	type Key: Clone + Hash + 'static;
}

/// Types that are used as values for a [`Node`].
pub trait IsExpr<'a, T: IsNode + 'a>: 'a + Copy + Debug + Send + Sync {
	fn children(&self) -> NodeIterator<'a, T>;
}

//====================================================================================================================//
// Node
//====================================================================================================================//

#[derive(Copy, Clone)]
pub struct Node<'a, T: IsNode> {
	data: *const NodeData<'a, T>,
}

impl<'a, T: IsNode> Node<'a, T> {
	pub fn expr(&self) -> &'a T::Expr<'a> {
		self.data().expr()
	}

	pub fn key(&self) -> T::Key {
		todo!()
	}

	pub fn parent(&self) -> Option<Node<'a, T>> {
		todo!()
	}

	pub fn next(&self) -> Option<Node<'a, T>> {
		todo!()
	}

	pub fn prev(&self) -> Option<Node<'a, T>> {
		todo!()
	}

	pub fn children(&self) -> NodeIterator<'a, T> {
		todo!()
	}

	pub fn len(&self) -> usize {
		todo!()
	}

	fn data(&self) -> &'a NodeData<'a, T> {
		unsafe { &*self.data }
	}

	unsafe fn data_mut(&self) -> &'a mut NodeData<'a, T> {
		let data = self.data as *mut NodeData<'a, T>;
		&mut *data
	}
}

impl<'a, T: IsNode> PartialEq for Node<'a, T> {
	fn eq(&self, other: &Self) -> bool {
		self.data == other.data
	}
}

impl<'a, T: IsNode> Eq for Node<'a, T> {}

unsafe impl<'a, T: IsNode> Send for Node<'a, T> {}
unsafe impl<'a, T: IsNode> Sync for Node<'a, T> {}

impl<'a, T: IsNode> Debug for Node<'a, T> {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		self.expr().fmt(f)
	}
}

//====================================================================================================================//
// NodeList
//====================================================================================================================//

#[derive(Copy, Clone)]
pub union NodeList<'a, T: IsNode> {
	fix: NodeListFix<'a, T>,
	vec: NodeListVec<'a, T>,
}

#[derive(Copy, Clone)]
struct NodeListFix<'a, T: IsNode> {
	len: usize,
	ptr: [*const NodeData<'a, T>; 3],
}

#[derive(Copy, Clone)]
struct NodeListVec<'a, T: IsNode> {
	len: usize,
	ptr: *const *const NodeData<'a, T>,
}

impl<'a, T: IsNode> NodeList<'a, T> {
	pub const fn empty() -> Self {
		let null = std::ptr::null_mut();
		NodeList {
			fix: NodeListFix {
				len: 0,
				ptr: [null, null, null],
			},
		}
	}

	pub fn single(node: Node<'a, T>) -> Self {
		let node = node.data;
		let null = std::ptr::null_mut();
		NodeList {
			fix: NodeListFix {
				len: 1,
				ptr: [node, null, null],
			},
		}
	}

	pub fn pair(a: Node<'a, T>, b: Node<'a, T>) -> Self {
		let a = a.data;
		let b = b.data;
		let null = std::ptr::null_mut();
		NodeList {
			fix: NodeListFix {
				len: 2,
				ptr: [a, b, null],
			},
		}
	}

	pub fn triple(a: Node<'a, T>, b: Node<'a, T>, c: Node<'a, T>) -> Self {
		let a = a.data;
		let b = b.data;
		let c = c.data;
		NodeList {
			fix: NodeListFix { len: 3, ptr: [a, b, c] },
		}
	}

	#[inline(always)]
	pub fn len(&self) -> usize {
		unsafe { self.fix.len }
	}

	#[inline(always)]
	pub fn get(&self, index: usize) -> Option<Node<'a, T>> {
		let len = self.len();
		if index < len {
			let ptr = unsafe {
				if len <= self.fix_len() {
					self.fix.ptr[index]
				} else {
					self.vec.ptr.add(index).read()
				}
			};
			let node = Node { data: ptr };
			Some(node)
		} else {
			None
		}
	}

	pub fn iter(&self) -> NodeIterator<'a, T> {
		self.into_iter()
	}

	#[inline(always)]
	const fn fix_len(&self) -> usize {
		unsafe { self.fix.ptr.len() }
	}
}

impl<'a, T: IsNode> Default for NodeList<'a, T> {
	fn default() -> Self {
		Self::empty()
	}
}

impl<'a, T: IsNode> PartialEq for NodeList<'a, T> {
	fn eq(&self, other: &Self) -> bool {
		if self.len() == other.len() {
			for i in 0..self.len() {
				if self.get(i) != other.get(i) {
					return false;
				}
			}
			true
		} else {
			false
		}
	}
}

impl<'a, T: IsNode> Eq for NodeList<'a, T> {}

unsafe impl<'a, T: IsNode> Send for NodeList<'a, T> {}
unsafe impl<'a, T: IsNode> Sync for NodeList<'a, T> {}

impl<'a, T: IsNode> Debug for NodeList<'a, T> {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		// TODO: implement proper
		write!(f, "[")?;
		for (n, it) in self.iter().enumerate() {
			write!(f, "{}", if n == 0 { " " } else { ", " })?;
			write!(f, "{it:?}")?;
		}
		write!(f, " ]")
	}
}

/// Iterator over a [`NodeList`].
pub struct NodeIterator<'a, T: IsNode> {
	list: NodeList<'a, T>,
	next: usize,
}

impl<'a, T: IsNode> NodeIterator<'a, T> {
	pub fn empty() -> Self {
		NodeList::empty().into_iter()
	}

	pub fn single(node: &Node<'a, T>) -> Self {
		NodeList::single(*node).into_iter()
	}

	pub fn to_list(&self) -> NodeList<'a, T> {
		self.list
	}
}

impl<'a, T: IsNode> Iterator for NodeIterator<'a, T> {
	type Item = Node<'a, T>;

	fn next(&mut self) -> Option<Self::Item> {
		let next = self.list.get(self.next);
		if next.is_some() {
			self.next += 1;
		}
		next
	}
}

impl<'a, T: IsNode> IntoIterator for NodeList<'a, T> {
	type Item = Node<'a, T>;
	type IntoIter = NodeIterator<'a, T>;

	fn into_iter(self) -> Self::IntoIter {
		NodeIterator { list: self, next: 0 }
	}
}

//====================================================================================================================//
// NodeStore
//====================================================================================================================//

pub struct NodeStore<T: IsNode> {
	buffer: Buffer,
	nodes: RawArena,
	_expr: PhantomData<T>,
}

impl<T: IsNode> NodeStore<T> {
	pub fn new<'a>() -> Self {
		Self {
			buffer: Buffer::default(),
			nodes: RawArena::for_type::<NodeData<T>>(1024),
			_expr: Default::default(),
		}
	}
}

pub struct NodeSet<'a, T: IsNode> {
	data: &'a NodeStore<T>,
	_expr: PhantomData<T>,
}

impl<'a, T: IsNode> NodeSet<'a, T> {
	pub fn new(store: &'a NodeStore<T>) -> Self {
		assert!(!std::mem::needs_drop::<NodeData<T>>());
		Self {
			data: store,
			_expr: Default::default(),
		}
	}

	pub fn new_node(&self, expr: T::Expr<'a>) -> Node<'a, T> {
		let data = self.data.nodes.push(NodeData::new(expr));
		Node { data }
	}

	pub fn list_empty(&self) -> NodeList<'a, T> {
		NodeList::empty()
	}

	pub fn list_single(&self, node: Node<'a, T>) -> NodeList<'a, T> {
		NodeList::single(node)
	}

	pub fn list_pair(&self, a: Node<'a, T>, b: Node<'a, T>) -> NodeList<'a, T> {
		NodeList::pair(a, b)
	}

	pub fn list_triple(&self, a: Node<'a, T>, b: Node<'a, T>, c: Node<'a, T>) -> NodeList<'a, T> {
		NodeList::triple(a, b, c)
	}

	pub fn list_from(&self, nodes: &[Node<'a, T>]) -> NodeList<'a, T> {
		match nodes.len() {
			0 => self.list_empty(),
			1 => self.list_single(nodes[0]),
			2 => self.list_pair(nodes[0], nodes[1]),
			3 => self.list_triple(nodes[0], nodes[1], nodes[2]),
			_ => {
				let bytes = std::mem::size_of::<*const NodeData<'a, T>>() * nodes.len();
				let ptr = self.data.buffer.alloc(bytes) as *mut *const NodeData<'a, T>;
				let mut cur = ptr;
				for it in nodes.iter() {
					unsafe {
						cur.write(it.data);
						cur = cur.add(1);
					}
				}
				let vec = NodeListVec { len: nodes.len(), ptr };
				let _ = vec.len; // otherwise unused
				NodeList { vec }
			}
		}
	}
}

//====================================================================================================================//
// NodeData
//====================================================================================================================//

struct NodeData<'a, T: IsNode> {
	expr: T::Expr<'a>,
	version: AtomicU32,
	index: AtomicU32,
	parent: AtomicPtr<NodeData<'a, T>>,
}

#[allow(unused)]
impl<'a, T: IsNode> NodeData<'a, T> {
	pub fn new(expr: T::Expr<'a>) -> Self {
		Self {
			expr,
			version: Default::default(),
			index: Default::default(),
			parent: Default::default(),
		}
	}

	#[inline(always)]
	pub fn version(&self) -> u32 {
		self.version.load(Ordering::SeqCst)
	}

	#[inline(always)]
	pub fn inc_version(&mut self, version: u32) {
		let ok = self
			.version
			.compare_exchange(version, version + 1, Ordering::SeqCst, Ordering::SeqCst);
		ok.expect("Node data got dirty while changing");
	}

	pub fn expr(&self) -> &T::Expr<'a> {
		&self.expr
	}

	pub fn set_expr(&mut self, expr: T::Expr<'a>) {
		let version = self.version();

		// clear the parent for the old children nodes
		for it in self.expr().children() {
			let data = unsafe { it.data_mut() };
			data.index.store(0, Ordering::SeqCst);
			data.parent.store(std::ptr::null_mut(), Ordering::SeqCst);
		}

		// set the new expression
		self.expr = expr;

		// set the parent for the new expression
		for (index, it) in self.expr().children().enumerate() {
			let data = unsafe { it.data_mut() };
			data.index.store(index as u32, Ordering::SeqCst);
			data.parent.store(self, Ordering::SeqCst);
		}

		self.inc_version(version);
	}
}

//====================================================================================================================//
// Tests
//====================================================================================================================//

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_simple() {
		let data = NodeStore::<Test>::new();
		let store = NodeSet::new(&data);
		let list = make_simple_list(&store);

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
	}

	fn make_simple_list<'a>(store: &'a NodeSet<'a, Test>) -> Node<'a, Test> {
		let zero = store.new_node(TestExpr::Zero);
		let node = store.new_node(TestExpr::Node(zero));
		let list = store.new_node(TestExpr::List(NodeList::pair(zero, node)));
		list
	}

	fn make_compiler<'a>(data: &'a NodeStore<Test>) -> Compiler<'a> {
		let mut compiler = Compiler::new(data);
		compiler.add_zero();
		compiler
	}

	struct Compiler<'a> {
		store: NodeSet<'a, Test>,
		nodes: Vec<Node<'a, Test>>,
	}

	impl<'a> Compiler<'a> {
		pub fn new(store: &'a NodeStore<Test>) -> Self {
			let store = NodeSet::new(store);
			Self {
				store,
				nodes: Default::default(),
			}
		}

		pub fn add_zero(&mut self) -> Node<'a, Test> {
			let node = self.store.new_node(TestExpr::Zero);
			self.nodes.push(node);
			node
		}

		pub fn add_num(&mut self, value: i32) -> Node<'a, Test> {
			let node = self.store.new_node(TestExpr::Number(value));
			self.nodes.push(node);
			node
		}

		pub fn get(&self, index: usize) -> Node<'a, Test> {
			self.nodes[index]
		}
	}

	#[derive(Copy, Clone, Debug)]
	enum TestExpr<'a> {
		Zero,
		Node(Node<'a, Test>),
		List(NodeList<'a, Test>),
		Number(i32),
	}

	#[derive(Copy, Clone)]
	struct Test;

	impl IsNode for Test {
		type Expr<'a> = TestExpr<'a>;

		type Key = String;
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
