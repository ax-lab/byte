use super::*;

mod node_iter;
mod node_list;
mod node_set;
mod node_writer;

pub use node_iter::*;
pub use node_list::*;
pub use node_set::*;
pub use node_writer::*;

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
		self.expr().children()
	}

	pub fn len(&self) -> usize {
		self.expr().children().len()
	}

	/// Value incremented when the node or one of its properties changes.
	#[inline(always)]
	pub fn version(&self) -> usize {
		self.data().version()
	}

	#[inline(always)]
	pub(crate) fn data(&self) -> &'a NodeData<'a, T> {
		unsafe { &*self.data }
	}

	unsafe fn data_mut(&self) -> &'a mut NodeData<'a, T> {
		let data = self.data as *mut NodeData<'a, T>;
		&mut *data
	}

	//----------------------------------------------------------------------------------------------------------------//
	// Bindings
	//----------------------------------------------------------------------------------------------------------------//

	#[inline(always)]
	pub(crate) fn binding(&self) -> usize {
		self.data().binding()
	}

	#[inline(always)]
	pub(crate) fn replace_binding(&self, old_id: usize, new_id: usize) -> Success {
		unsafe { self.data_mut() }.replace_binding(old_id, new_id)
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
	version: AtomicUsize,
	index: AtomicUsize,
	parent: AtomicPtr<NodeData<'a, T>>,
	binding: AtomicUsize,
}

#[allow(unused)]
impl<'a, T: IsNode> NodeData<'a, T> {
	pub fn new(expr: T::Expr<'a>) -> Self {
		Self {
			expr,
			version: Default::default(),
			index: Default::default(),
			parent: Default::default(),
			binding: Default::default(),
		}
	}

	#[inline(always)]
	pub fn binding(&self) -> usize {
		self.binding.load(Ordering::SeqCst)
	}

	#[inline(always)]
	pub fn replace_binding(&mut self, old_id: usize, new_id: usize) -> Success {
		self.binding
			.compare_exchange(old_id, new_id, Ordering::SeqCst, Ordering::SeqCst)
			.success()
	}

	#[inline(always)]
	pub fn version(&self) -> usize {
		self.version.load(Ordering::SeqCst)
	}

	#[inline(always)]
	pub fn change_version(&mut self, actual_version: usize) -> Success {
		self.version
			.compare_exchange(actual_version, actual_version + 1, Ordering::SeqCst, Ordering::SeqCst)
			.success()
	}

	#[inline(always)]
	pub fn bump_version(&mut self) {
		self.version.fetch_add(1, Ordering::SeqCst);
	}

	pub fn index(&self) -> usize {
		self.index.load(Ordering::SeqCst)
	}

	pub fn expr(&self) -> &T::Expr<'a> {
		&self.expr
	}

	pub fn expr_mut(&mut self) -> &mut T::Expr<'a> {
		&mut self.expr
	}

	pub fn set_expr(&mut self, expr: T::Expr<'a>) {
		let version = self.version();

		// clear the parent for the old children nodes
		for it in self.expr().children() {
			let data = unsafe { it.data_mut() };
			data.index.store(0, Ordering::SeqCst);
			data.parent.store(std::ptr::null_mut(), Ordering::SeqCst);
			data.bump_version();
		}

		// set the new expression
		self.expr = expr;

		// set the parent for the new expression
		for (index, it) in self.expr().children().enumerate() {
			let data = unsafe { it.data_mut() };
			data.index.store(index, Ordering::SeqCst);
			data.parent.store(self, Ordering::SeqCst);
			data.bump_version();
		}

		self.change_version(version)
			.expect("set_expr: node version changed while updating");
	}
}
