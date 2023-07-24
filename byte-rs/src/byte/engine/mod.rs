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

pub mod arena;
use arena::*;

#[derive(Copy, Clone, Debug)]
pub struct Node<'a> {
	data: *mut NodeData<'a>,
}

impl<'a> Node<'a> {
	pub fn expr(&self) -> &'a Expr<'a> {
		self.data().expr()
	}

	pub fn key(&self) -> Key {
		todo!()
	}

	pub fn parent(&self) -> Option<Node> {
		todo!()
	}

	pub fn next(&self) -> Option<Node> {
		todo!()
	}

	pub fn prev(&self) -> Option<Node> {
		todo!()
	}

	pub fn children(&self) -> NodeIterator {
		todo!()
	}

	pub fn len(&self) -> usize {
		todo!()
	}

	fn data(&self) -> &'a NodeData<'a> {
		unsafe { &*self.data }
	}

	unsafe fn data_mut(&self) -> &'a mut NodeData<'a> {
		unsafe { &mut *self.data }
	}
}

impl<'a> PartialEq for Node<'a> {
	fn eq(&self, other: &Self) -> bool {
		self.data == other.data
	}
}

impl<'a> Eq for Node<'a> {}

unsafe impl<'a> Send for Node<'a> {}
unsafe impl<'a> Sync for Node<'a> {}

#[derive(Default, Debug, Clone, Copy)]
pub enum Expr<'a> {
	#[default]
	None,
	Group(Node<'a>),
	Line(NodeList<'a>),
}

impl<'a> Expr<'a> {
	pub fn children(&self) -> NodeIterator<'a> {
		let none = || NodeList::empty().into_iter();
		let one = |node: &Node<'a>| NodeList::single(*node).into_iter();
		let all = |list: &NodeList<'a>| (*list).into_iter();
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
pub union NodeList<'a> {
	fix: NodeListFix<'a>,
	vec: NodeListVec<'a>,
}

#[derive(Copy, Clone)]
struct NodeListFix<'a> {
	len: usize,
	ptr: [*mut NodeData<'a>; 3],
}

#[derive(Copy, Clone)]
struct NodeListVec<'a> {
	len: usize,
	ptr: *mut *mut NodeData<'a>,
}

impl<'a> NodeList<'a> {
	pub const fn empty() -> Self {
		let null = std::ptr::null_mut();
		NodeList {
			fix: NodeListFix {
				len: 0,
				ptr: [null, null, null],
			},
		}
	}

	pub fn single(node: Node<'a>) -> Self {
		let node = node.data;
		let null = std::ptr::null_mut();
		NodeList {
			fix: NodeListFix {
				len: 1,
				ptr: [node, null, null],
			},
		}
	}

	pub fn pair(a: Node<'a>, b: Node<'a>) -> Self {
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

	pub fn triple(a: Node<'a>, b: Node<'a>, c: Node<'a>) -> Self {
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
	pub fn get(&self, index: usize) -> Option<Node<'a>> {
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

	pub fn iter(&self) -> NodeIterator<'a> {
		self.into_iter()
	}

	#[inline(always)]
	const fn fix_len(&self) -> usize {
		unsafe { self.fix.ptr.len() }
	}
}

impl<'a> Default for NodeList<'a> {
	fn default() -> Self {
		Self::empty()
	}
}

impl<'a> PartialEq for NodeList<'a> {
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

impl<'a> Eq for NodeList<'a> {}

unsafe impl<'a> Send for NodeList<'a> {}
unsafe impl<'a> Sync for NodeList<'a> {}

impl<'a> Debug for NodeList<'a> {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		todo!()
	}
}

/// Iterator over a [`NodeList`].
pub struct NodeIterator<'a> {
	list: NodeList<'a>,
	next: usize,
}

impl<'a> NodeIterator<'a> {
	pub fn to_list(&self) -> NodeList<'a> {
		self.list
	}
}

impl<'a> Iterator for NodeIterator<'a> {
	type Item = Node<'a>;

	fn next(&mut self) -> Option<Self::Item> {
		let next = self.list.get(self.next);
		if next.is_some() {
			self.next += 1;
		}
		next
	}
}

impl<'a> IntoIterator for NodeList<'a> {
	type Item = Node<'a>;
	type IntoIter = NodeIterator<'a>;

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
	pub fn new() -> Self {
		assert!(!std::mem::needs_drop::<NodeData>());
		Self {
			buffer: Default::default(),
			nodes: RawArena::for_type::<NodeData>(1024),
		}
	}

	pub fn new_node<'a>(&'a self, expr: Expr<'a>) -> Node<'a> {
		todo!()
	}

	pub fn list_empty<'a>(&'a self) -> NodeList<'a> {
		NodeList::empty()
	}

	pub fn list_single<'a>(&'a self, node: Node<'a>) -> NodeList<'a> {
		NodeList::single(node)
	}

	pub fn list_pair<'a>(&'a self, a: Node<'a>, b: Node<'a>) -> NodeList<'a> {
		NodeList::pair(a, b)
	}

	pub fn list_triple<'a>(&'a self, a: Node<'a>, b: Node<'a>, c: Node<'a>) -> NodeList<'a> {
		NodeList::triple(a, b, c)
	}

	pub fn list_from<'a>(&'a self, nodes: &[Node<'a>]) -> NodeList<'a> {
		match nodes.len() {
			0 => self.list_empty(),
			1 => self.list_single(nodes[0]),
			2 => self.list_pair(nodes[0], nodes[1]),
			3 => self.list_triple(nodes[0], nodes[1], nodes[2]),
			_ => {
				let bytes = std::mem::size_of::<*mut NodeData<'a>>() * nodes.len();
				let ptr = self.buffer.alloc(bytes) as *mut *mut NodeData<'a>;
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

struct NodeData<'a> {
	expr: Expr<'a>,
	version: AtomicU32,
	index: AtomicU32,
	parent: AtomicPtr<NodeData<'a>>,
}

impl<'a> NodeData<'a> {
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

	pub fn expr(&self) -> &Expr<'a> {
		&self.expr
	}

	pub fn set_expr(&mut self, expr: Expr<'a>) {
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
		assert_safe::<NodeData>();
		assert_safe::<Node>();
		assert_safe::<Expr>();

		assert_copy::<Node>();
		assert_copy::<Expr>();
	}
};

#[cfg(test)]
mod tests {}
