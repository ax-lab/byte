#![allow(unused)]

use std::{
	collections::VecDeque,
	fmt::Debug,
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

//====================================================================================================================//
// Node
//====================================================================================================================//

#[derive(Copy, Clone)]
pub struct Node<'a, T: IsExpr<'a>> {
	data: *const NodeData<'a, T>,
}

impl<'a, T: IsExpr<'a>> Node<'a, T> {
	pub fn expr(&self) -> &'a T {
		self.data().expr()
	}

	pub fn key(&self) -> Key {
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

	pub fn children(&self) -> NodeIterator<'a, Expr<'a>> {
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

impl<'a, T: IsExpr<'a>> PartialEq for Node<'a, T> {
	fn eq(&self, other: &Self) -> bool {
		self.data == other.data
	}
}

impl<'a, T: IsExpr<'a>> Eq for Node<'a, T> {}

unsafe impl<'a, T: IsExpr<'a>> Send for Node<'a, T> {}
unsafe impl<'a, T: IsExpr<'a>> Sync for Node<'a, T> {}

impl<'a, T: IsExpr<'a>> Debug for Node<'a, T> {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		self.expr().fmt(f)
	}
}

//====================================================================================================================//
// Expr
//====================================================================================================================//

pub trait IsExpr<'a>: 'a + Copy + Debug + Send + Sync {
	fn children(&self) -> NodeIterator<'a, Self>;
}

#[derive(Default, Debug, Clone, Copy)]
pub enum Expr<'a> {
	#[default]
	None,
	Group(Node<'a, Expr<'a>>),
	Line(NodeList<'a, Expr<'a>>),
}

impl<'a> Expr<'a> {}

impl<'a> IsExpr<'a> for Expr<'a> {
	fn children(&self) -> NodeIterator<'a, Expr<'a>> {
		let none = || NodeList::empty().into_iter();
		let one = |node: &Node<'a, Expr<'a>>| NodeList::single(*node).into_iter();
		let all = |list: &NodeList<'a, Expr<'a>>| (*list).into_iter();
		match self {
			Expr::None => none(),
			Expr::Group(node) => one(node),
			Expr::Line(nodes) => all(nodes),
		}
	}
}

pub enum Key {}

//====================================================================================================================//
// NodeList
//====================================================================================================================//

#[derive(Copy, Clone)]
pub union NodeList<'a, T: IsExpr<'a>> {
	fix: NodeListFix<'a, T>,
	vec: NodeListVec<'a, T>,
}

#[derive(Copy, Clone)]
struct NodeListFix<'a, T: IsExpr<'a>> {
	len: usize,
	ptr: [*const NodeData<'a, T>; 3],
}

#[derive(Copy, Clone)]
struct NodeListVec<'a, T: IsExpr<'a>> {
	len: usize,
	ptr: *const *const NodeData<'a, T>,
}

impl<'a, T: IsExpr<'a>> NodeList<'a, T> {
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

impl<'a, T: IsExpr<'a>> Default for NodeList<'a, T> {
	fn default() -> Self {
		Self::empty()
	}
}

impl<'a, T: IsExpr<'a>> PartialEq for NodeList<'a, T> {
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

impl<'a, T: IsExpr<'a>> Eq for NodeList<'a, T> {}

unsafe impl<'a, T: IsExpr<'a>> Send for NodeList<'a, T> {}
unsafe impl<'a, T: IsExpr<'a>> Sync for NodeList<'a, T> {}

impl<'a, T: IsExpr<'a>> Debug for NodeList<'a, T> {
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
pub struct NodeIterator<'a, T: IsExpr<'a>> {
	list: NodeList<'a, T>,
	next: usize,
}

impl<'a, T: IsExpr<'a>> NodeIterator<'a, T> {
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

impl<'a, T: IsExpr<'a>> Iterator for NodeIterator<'a, T> {
	type Item = Node<'a, T>;

	fn next(&mut self) -> Option<Self::Item> {
		let next = self.list.get(self.next);
		if next.is_some() {
			self.next += 1;
		}
		next
	}
}

impl<'a, T: IsExpr<'a>> IntoIterator for NodeList<'a, T> {
	type Item = Node<'a, T>;
	type IntoIter = NodeIterator<'a, T>;

	fn into_iter(self) -> Self::IntoIter {
		NodeIterator { list: self, next: 0 }
	}
}

//====================================================================================================================//
// NodeStore
//====================================================================================================================//

pub struct NodeStore {
	buffer: Buffer,
	nodes: RawArena,
}

impl NodeStore {
	pub fn for_expr<'a, T: IsExpr<'a>>() -> Self {
		Self {
			buffer: Buffer::default(),
			nodes: RawArena::for_type::<NodeData<'a, T>>(1024),
		}
	}
}

pub struct NodeSet<'a, T: IsExpr<'a>> {
	data: &'a NodeStore,
	_expr: PhantomData<T>,
}

impl<'a, T: IsExpr<'a>> NodeSet<'a, T> {
	pub fn new(store: &'a NodeStore) -> Self {
		assert!(!std::mem::needs_drop::<NodeData<T>>());
		Self {
			data: store,
			_expr: Default::default(),
		}
	}

	pub fn new_node(&self, expr: T) -> Node<'a, T> {
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
				NodeList {
					vec: NodeListVec { len: nodes.len(), ptr },
				}
			}
		}
	}
}

//====================================================================================================================//
// NodeData
//====================================================================================================================//

struct NodeData<'a, T: IsExpr<'a>> {
	expr: T,
	version: AtomicU32,
	index: AtomicU32,
	parent: AtomicPtr<NodeData<'a, T>>,
}

impl<'a, T: IsExpr<'a>> NodeData<'a, T> {
	pub fn new(expr: T) -> Self {
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

	pub fn expr(&self) -> &T {
		&self.expr
	}

	pub fn set_expr(&mut self, expr: T) {
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

const _: () = {
	fn assert_safe<T: Send + Sync>() {}
	fn assert_copy<T: Copy>() {}

	fn assert_all() {
		assert_safe::<NodeData<Expr>>();
		assert_safe::<Node<Expr>>();
		assert_safe::<Expr>();

		assert_copy::<Node<Expr>>();
		assert_copy::<Expr>();
	}
};

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_simple() {
		let data = NodeStore::for_expr::<Val<'_>>();
		let store = NodeSet::new(&data);
		let list = make_simple_list(&store);

		let actual = format!("{list:?}");
		assert_eq!(actual, "List([ Zero, Node(Zero) ])");
	}

	#[test]
	fn test_compiler() {
		let data = NodeStore::for_expr::<Val<'_>>();
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

	fn make_simple_list<'a>(store: &'a NodeSet<'a, Val<'a>>) -> Node<'a, Val<'a>> {
		let zero = store.new_node(Val::Zero);
		let node = store.new_node(Val::Node(zero));
		let list = store.new_node(Val::List(NodeList::pair(zero, node)));
		list
	}

	fn make_compiler<'a>(data: &'a NodeStore) -> Compiler<'a> {
		let mut compiler = Compiler::new(data);
		compiler.add_zero();
		compiler
	}

	struct Compiler<'a> {
		store: NodeSet<'a, Val<'a>>,
		nodes: RwLock<Vec<Node<'a, Val<'a>>>>,
	}

	impl<'a> Compiler<'a> {
		pub fn new(store: &'a NodeStore) -> Self {
			let store = NodeSet::new(store);
			Self {
				store,
				nodes: Default::default(),
			}
		}

		pub fn add_zero(&self) -> Node<'a, Val<'a>> {
			let node = self.store.new_node(Val::Zero);
			let mut nodes = self.nodes.write().unwrap();
			nodes.push(node);
			node
		}

		pub fn add_num(&self, value: i32) -> Node<'a, Val<'a>> {
			let node = self.store.new_node(Val::Number((value)));
			let mut nodes = self.nodes.write().unwrap();
			nodes.push(node);
			node
		}

		pub fn get(&self, index: usize) -> Node<'a, Val<'a>> {
			self.nodes.read().unwrap()[index]
		}
	}

	#[derive(Copy, Clone, Debug)]
	enum Val<'a> {
		Zero,
		Node(Node<'a, Val<'a>>),
		List(NodeList<'a, Val<'a>>),
		Number(i32),
	}

	impl<'a> IsExpr<'a> for Val<'a> {
		fn children(&self) -> NodeIterator<'a, Val<'a>> {
			match self {
				Val::Zero => NodeIterator::empty(),
				Val::Node(node) => NodeIterator::single(node),
				Val::List(list) => list.into_iter(),
				Val::Number(..) => NodeIterator::empty(),
			}
		}
	}
}
