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

	pub fn remove_node(&mut self, node: Node<'a, T>) {
		self.removed.push((node.version(), node));
	}

	// NOTE: nodes may be moved by setting the values of a node

	/*

		Algorithm
		=========

		1) Flag nodes in deleted node ranges for deletion

		2) Remove deletion flag from nodes marked for keeping

		3) Process moved and replaced ranges:

		- ranges must not partially overlap;
		- a range must be under a single parent;
		- the same range must not be moved or replaced twice;
		- the anchor node for the move must be outside the range;
		- nested moves are allowed;

			NOTE: This can also be used as ranged remove by "moving" a range
			to "the void".

			## Implementation details

			a) group updates by parent
			b) for each parent: sort by `index` then `length`
			c) validate for partial overlaps
			d) apply updates _bottom up_ using the sorted order

				dx) replace nodes "in place", incrementing the incoming nodes'
					move count

				dy) remove moved nodes, increment their moved count and move
					to their new parent

				dz) the move target anchor follows the same logic as inserts

		4) Process updates, flag new child nodes for moving

		5) Process non-linked inserts

		6) Delete nodes marked for deletion

		7) Process linked inserts

		8) Actually move updated children

		X) Check post-validations:

			- Validate that any node was moved at most ONCE

	*/

	pub fn update_node(&mut self, node: &Node<'a, T>, new_value: T::Expr<'a>) {
		/*
			Details:

			- a node can only be updated once
			- the new value's children are implicitly moved to the node
			- old value's children are "forgotten" (but are free to move)
			- updates are tricky because of that, as they can implicitly move nodes
			- any given node can only be moved to a single parent node
		*/
		self.updated.push((node.version(), node.key(), *node, new_value));
	}

	pub fn insert_range<I: IntoIterator<Item = Node<'a, T>>>(&mut self, anchor: Anchor<'a, T>, nodes: I) {
		/*
			Details:

			Insert a range of nodes in reference to the given anchor nodes.

			The insert "follows" the node around if it's moved. Linked inserts
			are also removed if their anchor is removed.

			Requirements:

			- inserted range must not be in the tree
			- multiple inserts with the same anchor are not allowed
		*/
		let _ = (anchor, nodes);
		todo!()
	}

	pub fn remove_range(&mut self, from: &Node<'a, T>, to: &Node<'a, T>) {
		/*
			Details:

			A remove operation will flag the range of nodes for deletion. The
			actual deletion happens after processing other tree updates.

			Because nodes are just flagged before deletion, the delete does not
			conflict with other operations (e.g. even updates, however dubious).

			Moving deleted nodes is allowed, but they will still get deleted.

			Note that remove insert anchors will also extend to linked nodes.
		*/
		assert!(from.parent() == to.parent());
		todo!()
	}

	pub fn replace_range<I: IntoIterator<Item = Node<'a, T>>>(
		&mut self,
		from: &Node<'a, T>,
		to: &Node<'a, T>,
		nodes: I,
	) {
		let _ = (from, to, nodes);
		todo!()
	}

	pub fn move_range(&mut self, from: &Node<'a, T>, to: &Node<'a, T>, at: Anchor<'a, T>) {
		let _ = (from, to, at);
		todo!()
	}

	pub fn keep_range(&mut self, from: &Node<'a, T>, to: &Node<'a, T>) {
		/*
			Details:

			This will clear the delete flag of any node that was flagged as
			deleted, preventing its removal.
		*/
		let _ = (from, to);
		todo!()
	}
}

pub enum Anchor<'a, T: IsNode> {
	Before(Node<'a, T>),
	After(Node<'a, T>),
	LinkedBefore(Node<'a, T>),
	LinkedAfter(Node<'a, T>),
}
