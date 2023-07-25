use super::*;

/// Manages a collection of [`Node`].
pub struct NodeSet<'a, T: IsNode> {
	store: &'a NodeStore<T>,
	_expr: PhantomData<T>,
}

impl<'a, T: IsNode> NodeSet<'a, T> {
	pub fn new(store: &'a NodeStore<T>) -> Self {
		assert!(!std::mem::needs_drop::<NodeData<T>>());
		Self {
			store,
			_expr: Default::default(),
		}
	}

	pub fn store(&self) -> &'a NodeStore<T> {
		self.store
	}

	pub fn new_node(&self, expr: T::Expr<'a>) -> Node<'a, T> {
		let data = self.store.nodes.push(NodeData::new(expr));
		Node { data }
	}

	pub fn list_from(&self, nodes: &[Node<'a, T>]) -> NodeList<'a, T> {
		match nodes.len() {
			0 => NodeList::empty(),
			1 => NodeList::single(nodes[0]),
			2 => NodeList::pair(nodes[0], nodes[1]),
			3 => NodeList::triple(nodes[0], nodes[1], nodes[2]),
			_ => NodeList::from_list(&self.store, nodes),
		}
	}
}
