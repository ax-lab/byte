use super::*;

/// Manages a collection of [`Node`].
pub struct NodeSet<'a, T: IsNode> {
	store: &'a NodeStore<T>,
	bindings: ScopeMap<'a, T>,
}

impl<'a, T: IsNode> NodeSet<'a, T> {
	pub fn new(store: &'a NodeStore<T>) -> Self {
		let bindings = ScopeMap::new();
		Self { store, bindings }
	}

	pub fn store(&self) -> &'a NodeStore<T> {
		self.store
	}

	pub fn new_node(&mut self, expr: T::Expr<'a>) -> Node<'a, T> {
		let data = self.store.nodes.push(NodeData::new(expr));
		let node = Node { data };
		self.add_node(node);
		node
	}

	pub fn bind(&mut self, key: T::Key, scope: Scope, data: T::Val) {
		self.bindings.bind(key, scope, data);
	}

	pub fn list_from(&self, nodes: &[Node<'a, T>]) -> NodeList<'a, T> {
		NodeList::from_list(&self.store, nodes)
	}

	pub fn resolve(&mut self) {
		while let Some(next) = self.bindings.shift_next() {
			println!("=> Shifted:");
			for (val, nodes) in next {
				println!("- apply {val:?} to {nodes:?}");
			}
		}
		todo!()
	}

	pub(crate) fn add_node(&mut self, node: Node<'a, T>) {
		self.bindings.add_node(node);
	}

	pub fn apply(&mut self, changes: ChangeSet<'a, T>) {
		// TODO: improve conflict reporting by tracing the root in the ChangeSet

		// TODO: some conflicts might be bogus, so use version as a first pass filter, but actually check for conflicts

		for node in changes.created {
			self.bindings.add_node(node);
		}

		for (version, old_key, node, new_value) in changes.updated {
			let data = unsafe { node.data_mut() };
			data.change_version(version).expect("node was updated multiple times");
			data.set_expr(new_value);
			self.bindings.reindex_node(node, &old_key);
		}

		for (version, node) in changes.removed {
			let data = unsafe { node.data_mut() };
			data.change_version(version).expect("removed node was changed");
			self.bindings.remove_node(node);
			if let Some(parent) = node.parent() {
				let parent = unsafe { parent.data_mut() };
				let parent_version = parent.version();
				let index = data.index();
				let children = parent.expr_mut().children_mut();
				if let Some(children) = children {
					assert!(children.get(index).unwrap() == node);
					*children = children.remove(self.store, index);
					parent.change_version(parent_version).expect("parent was changed");
				}
			}
			data.parent.store(std::ptr::null_mut(), Ordering::SeqCst);
		}
	}
}

//====================================================================================================================//
// ChangeSet
//====================================================================================================================//

pub struct ChangeSet<'a, T: IsNode> {
	store: &'a NodeStore<T>,
	created: Vec<Node<'a, T>>,
	updated: Vec<(usize, T::Key, Node<'a, T>, T::Expr<'a>)>,
	removed: Vec<(usize, Node<'a, T>)>,
}

impl<'a, T: IsNode> ChangeSet<'a, T> {
	pub fn create_node(&mut self, expr: T::Expr<'a>) -> Node<'a, T> {
		let data = self.store.nodes.push(NodeData::new(expr));
		let node = Node { data };
		self.created.push(node);
		node
	}

	pub fn update_node(&mut self, node: &Node<'a, T>, new_value: T::Expr<'a>) {
		self.updated.push((node.version(), node.key(), *node, new_value));
	}

	pub fn remove_node(&mut self, node: Node<'a, T>) {
		self.removed.push((node.version(), node));
	}
}
