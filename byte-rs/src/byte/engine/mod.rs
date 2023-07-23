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

	fn data(&self) -> &NodeData<'a> {
		unsafe { &*self.data }
	}

	unsafe fn data_mut(&self) -> &mut NodeData<'a> {
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

#[derive(Default, Debug)]
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

#[derive(Default)]
pub struct NodeStore {
	buffer: Buffer,
}

impl NodeStore {
	pub fn new() -> Self {
		Self::default()
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
	expr: AtomicPtr<Expr<'a>>,
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

	pub fn expr(&self) -> &'a Expr<'a> {
		let expr = self.expr.load(Ordering::SeqCst);
		if expr.is_null() {
			static EMPTY: Expr = Expr::None;
			unsafe { std::mem::transmute(&EMPTY) }
		} else {
			unsafe { &*expr }
		}
	}

	pub fn set_expr(&mut self, expr: *mut Expr<'a>) {
		let version = self.version();

		// clear the parent for the old children nodes
		for it in self.expr().children() {
			let data = unsafe { it.data_mut() };
			data.index.store(0, Ordering::SeqCst);
			data.parent.store(std::ptr::null_mut(), Ordering::SeqCst);
		}

		// set the new expression
		self.expr.store(expr, Ordering::SeqCst);

		// set the parent for the new expression
		for it in self.expr().children() {
			let data = unsafe { it.data_mut() };
			data.index.store(0, Ordering::SeqCst);
			data.parent.store(std::ptr::null_mut(), Ordering::SeqCst);
		}

		self.inc_version(version);
	}
}

//====================================================================================================================//
// Buffer
//====================================================================================================================//

#[derive(Default)]
pub struct Buffer {
	list_8: Arena<[u8; 8]>,
	list_16: Arena<[u8; 16]>,
	list_32: Arena<[u8; 32]>,
	list_64: Arena<[u8; 64]>,
	list_128: Arena<[u8; 128]>,
	large: RwLock<VecDeque<Vec<u8>>>,
}

impl Buffer {
	pub fn alloc(&self, size: usize) -> *mut u8 {
		unsafe {
			if size <= 8 {
				let data = std::mem::MaybeUninit::uninit();
				let data = self.list_8.push(data.assume_init());
				data as *mut u8
			} else if size <= 16 {
				let data = std::mem::MaybeUninit::uninit();
				let data = self.list_16.push(data.assume_init());
				data as *mut u8
			} else if size <= 32 {
				let data = std::mem::MaybeUninit::uninit();
				let data = self.list_32.push(data.assume_init());
				data as *mut u8
			} else if size <= 64 {
				let data = std::mem::MaybeUninit::uninit();
				let data = self.list_64.push(data.assume_init());
				data as *mut u8
			} else if size <= 128 {
				let data = std::mem::MaybeUninit::uninit();
				let data = self.list_128.push(data.assume_init());
				data as *mut u8
			} else {
				let mut data = Vec::with_capacity(size);
				let data_ptr = data.as_mut_ptr();
				let mut large = self.large.write().unwrap();
				large.push_back(data);
				data_ptr
			}
		}
	}

	pub fn push<T: Copy>(&self, value: T) -> *mut T {
		let size = std::mem::size_of::<T>();
		let data = self.alloc(size) as *mut T;
		unsafe {
			data.write(value);
			data
		}
	}
}

//====================================================================================================================//
// Arena
//====================================================================================================================//

pub struct Arena<T> {
	pages: RwLock<Vec<ArenaPage<T>>>,
	current: AtomicPtr<ArenaPage<T>>,
}

struct ArenaPage<T> {
	data: *mut T,
	size: usize,
	next: AtomicUsize,
}

impl<T> Drop for ArenaPage<T> {
	fn drop(&mut self) {
		let next = self.next.load(Ordering::SeqCst);
		let next = std::cmp::min(next, self.size); // next can go past size in store
		let data = unsafe { Vec::from_raw_parts(self.data, next, self.size) };
		drop(data);
	}
}

impl<T> Arena<T> {
	pub fn new() -> Self {
		Self {
			pages: Default::default(),
			current: Default::default(),
		}
	}

	pub fn push(&self, new: T) -> *mut T {
		loop {
			let ptr = self.current.load(Ordering::SeqCst);
			let ptr = if ptr.is_null() {
				let mut pages = self.pages.write().unwrap();
				let last = self.current.load(Ordering::SeqCst);
				if last.is_null() {
					let data = Vec::<T>::with_capacity(256);
					let size = data.capacity();
					let next = AtomicUsize::new(0);

					let mut data = ManuallyDrop::new(data);
					let data = data.as_mut_ptr();
					let page = ArenaPage { data, size, next };
					pages.push(page);
					let page = pages.last_mut().unwrap() as *mut ArenaPage<T>;
					self.current.store(page, Ordering::SeqCst);
					page
				} else {
					last
				}
			} else {
				ptr
			};
			let mut page = unsafe { NonNull::new_unchecked(ptr) };
			let page = unsafe { page.as_mut() };
			let next = page.next.fetch_add(1, Ordering::SeqCst);
			if next < page.size {
				unsafe {
					let data = page.data.add(next);
					data.write(new);
					return data;
				}
			} else {
				let _ = self
					.current
					.compare_exchange(ptr, std::ptr::null_mut(), Ordering::SeqCst, Ordering::SeqCst);
			}
		}
	}
}

impl<T> Default for Arena<T> {
	fn default() -> Self {
		Self::new()
	}
}

//====================================================================================================================//
// Tests
//====================================================================================================================//

const _: () = {
	fn assert_safe<T: Send + Sync>() {}

	fn assert_all() {
		assert_safe::<NodeData>();
		assert_safe::<Node>();
		assert_safe::<Expr>();
	}
};

#[cfg(test)]
mod tests {
	use std::{sync::Arc, time::Instant};

	use super::*;

	#[test]
	fn arena_simple() {
		let mut arena = Arena::new();
		let mut pointers = Vec::new();
		for i in 0..2048 {
			let ptr = arena.push(i);
			pointers.push(ptr);
		}

		assert_eq!(pointers.len(), 2048);
		for (n, ptr) in pointers.into_iter().enumerate() {
			let value = unsafe { ptr.as_ref().unwrap() };
			assert_eq!(value, &n);
		}
	}

	#[test]
	fn arena_drop() {
		let counter = Arc::new(RwLock::new(0));

		let mut arena = Arena::new();
		for _ in 0..2048 {
			arena.push(Value::new(counter.clone()));
		}

		assert_eq!(*counter.read().unwrap(), 2048);
		drop(arena);
		assert_eq!(*counter.read().unwrap(), 0);

		struct Value(Arc<RwLock<i32>>);

		impl Value {
			pub fn new(counter: Arc<RwLock<i32>>) -> Self {
				{
					let mut counter = counter.write().unwrap();
					*counter += 1;
				}
				Self(counter)
			}
		}

		impl Drop for Value {
			fn drop(&mut self) {
				let counter = &mut self.0;
				let mut counter = counter.write().unwrap();
				*counter -= 1;
			}
		}
	}

	#[allow(unused)]
	fn arena_benchmark() {
		let now = Instant::now();
		let mut arena = Arena::new();
		let mut counter = 0;
		for i in 0..2_000_000 {
			arena.push(i);
			counter += 1;
		}

		let elapsed = now.elapsed();
		let average = elapsed / counter;
		println!("stored {counter} items in {elapsed:?} (avg: {average:?})");
	}
}
