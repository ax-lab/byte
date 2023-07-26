use std::collections::HashMap;

use super::*;

/// Manages a collection of [`Node`].
pub struct NodeSet<'a, T: IsNode> {
	store: &'a NodeStore<T>,
	bindings: BindingMap<'a, T, ()>,
}

impl<'a, T: IsNode> NodeSet<'a, T> {
	pub fn new(store: &'a NodeStore<T>) -> Self {
		let bindings = BindingMap::new();
		Self { store, bindings }
	}

	pub fn store(&self) -> &'a NodeStore<T> {
		self.store
	}

	pub fn new_node(&mut self, expr: T::Expr<'a>) -> Node<'a, T> {
		let data = self.store.nodes.push(NodeData::new(expr));
		let node = Node { data };
		let key = node.key();
		let ptr = node.ptr();
		self.bindings.add_node(key, node.offset(), ptr);
		node
	}

	pub fn bind(&mut self, key: T::Key, scope: Scope, data: ()) {
		self.bindings.bind(key, scope, data, Override::InnerOnly);
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

pub enum Scope {
	Root,
	Range(usize, usize),
}

/// Defines the behavior of [`BindingMap`] when the specified scope overlaps
/// existing binds.
pub enum Override {
	/// Override the bind for the given scope only if it is fully contained
	/// in the existing scope.
	///
	/// Partially overlapping scopes will set the bind for the non-overlapping
	/// ranges, but will keep previous bindings.
	Inner,

	/// Like [`Override::Inner`] but panics if the ranges overlap partially.
	InnerOnly,

	/// Binds to [`Override::Inner`], but will also override partially
	/// overlapping ranges.
	///
	/// This will not override the value if the previous scope is contained
	/// in the new scope.
	InnerAndOverlap,

	/// Override all overlapping scopes.
	All,
}

pub(crate) struct BindingMap<'a, T: IsNode, U> {
	table: HashMap<T::Key, ScopeTree<'a, T, U>>,
}

impl<'a, T: IsNode, U> BindingMap<'a, T, U> {
	pub fn new() -> Self {
		Self {
			table: Default::default(),
		}
	}

	/// Set the binding for a key in the given scope.
	///
	/// Scopes are "nested". Smaller scopes override binds from the larger
	/// encompassing scope.
	///
	/// In practice, the above means that the full scope range for a given
	/// key is partitioned when a sub-range is bound.
	///
	/// If scope binds overlap, the behavior is defined by the [`Override`]
	/// flag.
	pub fn bind(&mut self, key: T::Key, scope: Scope, data: U, mode: Override) -> bool {
		let entry = self.table.entry(key).or_insert_with(|| ScopeTree::new());
		entry.bind(scope, data, mode)
	}

	pub fn add_node(&mut self, key: T::Key, offset: usize, node: *const NodeData<'a, T>) {
		let entry = self.table.entry(key).or_insert_with(|| ScopeTree::new());
		entry.add_node(offset, node)
	}
}

/// A scope tree for a specific key in a [`BindingMap`].
struct ScopeTree<'a, T: IsNode, U> {
	_data: PhantomData<&'a (T, U)>,
}

impl<'a, T: IsNode, U> ScopeTree<'a, T, U> {
	pub fn new() -> Self {
		todo!()
	}

	pub fn bind(&mut self, scope: Scope, data: U, mode: Override) -> bool {
		/*
			Initially a scope contains only a Scope::Root with no value. That
			scope is handled separately.

			For other scopes, check for overlapping scopes. If there is no
			overlap, just add the new scope with the value.

			Add new scope ranges for any non-overlapping ranges.

			TODO: should we break the ranges or keep them intact and use an
			actual tree for values?

			For each overlapping range:

			1) If new is an inner range, break outer in 2-3 ranges and set the
			   overlapping region to the new value.

			2) If new is an outer range, either override if `Override::All` or
			   do nothing. The outer parts will already be covered by (1).

			3) If the ranges overlap partially then:

				a) If `Override::InnerOnly` then panic;

				b) If `Override::Inner` do nothing. The non overlapping parts
				   are already covered by (1).

				c) Otherwise break the intersecting part from the existing
				   binding and override its value.

			4) TODO: what if the ranges are exactly equal? Overwrite the
			   value in case (3.c)?

			Note that breaking a scope range means splitting the nodes in it.
		*/
		todo!()
	}

	pub fn add_node(&mut self, offset: usize, node: *const NodeData<'a, T>) {
		/*
			Find the most specific range that contains the offset and add the
			node to it. Otherwise add it to the root.

			TODO: implement a "gap buffer" with lazy-sorting for keeping the nodes.
		*/
		todo!()
	}
}
