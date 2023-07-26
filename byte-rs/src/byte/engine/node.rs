use super::*;

mod node_iter;
mod node_list;
mod node_set;
pub use node_iter::*;
pub use node_list::*;
pub use node_set::*;

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
		self.expr().key()
	}

	pub fn offset(&self) -> usize {
		self.expr().offset()
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

	pub(crate) fn data(&self) -> &'a NodeData<'a, T> {
		unsafe { &*self.data }
	}

	pub(crate) fn ptr(&self) -> *const NodeData<'a, T> {
		self.data
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
// NodeData
//====================================================================================================================//

pub(crate) struct NodeData<'a, T: IsNode> {
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
