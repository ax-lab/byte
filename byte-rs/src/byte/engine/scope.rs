use std::collections::HashMap;

use super::*;

/// Represents a scope in the program.
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Scope {
	Root,
	Range(usize, usize),
}

//====================================================================================================================//
// ScopeMap
//====================================================================================================================//

/// Maps an [`IsNode::Key`] to an [`IsNode::Val`] in a given [`Scope`].
pub struct ScopeMap<'a, T: IsNode> {
	table: HashMap<T::Key, ScopeTree<'a, T>>,
	values: ValueTable<'a, T>,
}

impl<'a, T: IsNode> ScopeMap<'a, T> {
	pub fn new() -> Self {
		Self {
			table: Default::default(),
			values: ValueTable::new(),
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
	pub fn bind(&mut self, key: T::Key, scope: Scope, data: T::Val) {
		let entry = self
			.table
			.entry(key)
			.or_insert_with(|| ScopeTree::new(&mut self.values));
		entry.bind(&mut self.values, scope, data)
	}

	pub fn get(&self, key: &T::Key, offset: usize) -> Option<&T::Val> {
		if let Some(entry) = self.table.get(key) {
			entry.get(&self.values, offset)
		} else {
			None
		}
	}

	/// Add a node to the binding map. The node is associated with its key
	/// based on its offset.
	pub fn add_node(&mut self, node: Node<'a, T>) {
		let key = node.key();
		let entry = self
			.table
			.entry(key)
			.or_insert_with(|| ScopeTree::new(&mut self.values));
		entry.add_node(&mut self.values, node)
	}

	/// Update a node position in the binding map. This should be called
	/// whenever the node changes key.
	pub fn reindex_node(&mut self, node: Node<'a, T>, old_key: &T::Key) {
		let values = &mut self.values;
		let key = node.key();
		if &key != old_key {
			if let Some(entry) = self.table.get_mut(old_key) {
				entry.remove_node(values, &node);
			}
			self.add_node(node);
		}
	}

	/// Update a node position in the binding map. This should be called
	/// whenever the node changes key.
	pub fn remove_node(&mut self, node: Node<'a, T>) {
		let values = &mut self.values;
		let key = node.key();
		if let Some(entry) = self.table.get_mut(&key) {
			entry.remove_node(values, &node);
		}
	}

	/// Shifts the next set of nodes based on precedence of their bound key
	/// value.
	pub fn shift_next(&mut self) -> Option<ScopeValueIterator<'a, '_, T>> {
		self.values.shift_next()
	}
}

//----------------------------------------------------------------------------------------------------------------//
// ValueTable and Entry
//----------------------------------------------------------------------------------------------------------------//

/// Stores an entry for each value and node segment pair in the scope.
struct ValueTable<'a, T: IsNode> {
	list: Vec<ValueEntry<'a, T>>,
	heap_to_list: Vec<usize>, // list index at the given heap position
	list_to_heap: Vec<usize>, // heap position for the given list index
	heap_sorted: usize,       // number of sorted heap entries
	heap_length: usize,       // total number of non-removed heap entries
}

impl<'a, T: IsNode> ValueTable<'a, T> {
	pub fn new() -> Self {
		Self {
			list: Default::default(),
			heap_to_list: Default::default(),
			list_to_heap: Default::default(),
			heap_sorted: 0,
			heap_length: 0,
		}
	}

	pub fn new_entry(&mut self, value: T::Val, nodes: BoundNodes<'a, T>) -> usize {
		let index = self.list.len();
		self.list.push(ValueEntry { value, nodes });

		// add the entry to the unsorted section of the heap
		self.heap_to_list.push(index);
		self.list_to_heap.push(index);
		if index > self.heap_length {
			// make sure unsorted heap entries are contiguous, as there may be
			// removed ones
			self.heap_swap(index, self.heap_length);
		}
		self.heap_length += 1;

		index
	}

	pub fn extract_range(&mut self, index: usize, sta: usize, end: usize) -> BoundNodes<'a, T> {
		self.list[index].nodes.extract_range(sta, end)
	}

	pub fn add_node(&mut self, index: usize, node: Node<'a, T>) {
		self.list[index].nodes.add_node(node);
	}

	pub fn remove_node(&mut self, index: usize, node: &Node<'a, T>) {
		self.list[index].nodes.remove_node(node);
	}

	pub fn get_value(&self, index: usize) -> &T::Val {
		&self.list[index].value
	}

	pub fn set_value(&mut self, index: usize, value: T::Val) {
		self.list[index].value = value;
		self.heap_fixup(self.list_to_heap[index], false);
	}

	//------------------------------------------------------------------------------------------------------------//
	// Priority queue
	//------------------------------------------------------------------------------------------------------------//

	pub fn shift_next(&mut self) -> Option<ScopeValueIterator<'a, '_, T>> {
		if self.heap_sorted < self.heap_length {
			self.heapify();
		}

		if self.heap_length > 0 {
			let mut should_continue = true;
			let mut count = 0;
			let value = self.heap_value(0);
			while should_continue {
				self.list[self.heap_to_list[0]].nodes.fix_nodes();
				count += 1;
				self.heap_swap(0, self.heap_length - 1);
				self.heap_length -= 1;
				self.heap_sorted -= 1;
				let next_value = self.heap_value(0);
				should_continue = self.heap_length > 0 && next_value == value;
				self.heap_shift_down(0, next_value, self.heap_length);
			}

			Some(ScopeValueIterator {
				parent: self,
				next: self.heap_length,
				last: self.heap_length + count,
			})
		} else {
			None
		}
	}

	//------------------------------------------------------------------------------------------------------------//
	// Priority queue implementation
	//------------------------------------------------------------------------------------------------------------//

	/// Rebuild the heap invariant for all entries in the table.
	pub fn heapify(&mut self) {
		let entries = self.heap_length;
		let added = entries - self.heap_sorted;
		if added == 0 {
			return; // heap is already well formed
		}

		// check if we are better off rebuilding the heap with O(n) or
		// inserting the remaining elements with O(n * log n)
		let rebuild = if self.heap_sorted > 0 {
			let log_n = (usize::BITS - entries.leading_zeros()) as usize;
			entries / added <= log_n
		} else {
			true
		};

		if !rebuild {
			// fix the heap by inserting each of the new entries by shifting
			// them up into the heap from the bottom of the array
			while self.heap_sorted < entries {
				let next = self.heap_sorted;
				self.heap_sorted += 1;
				self.heap_fixup(next, true);
			}
		} else {
			// rebuild the entire heap from scratch using the bottom up method
			for pos in 0..=(entries - 1) / 2 {
				let val = self.heap_value(pos);
				self.heap_shift_down(pos, val, entries);
			}
			self.heap_sorted = entries;
		}
	}

	/// Precedence value for the entry at the given heap position.
	#[inline(always)]
	fn heap_value(&self, pos: usize) -> T::Precedence {
		T::get_precedence(&self.list[self.heap_to_list[pos]].value)
	}

	/// Swap two heap positions.
	fn heap_swap(&mut self, pos_a: usize, pos_b: usize) {
		let idx_a = self.heap_to_list[pos_a];
		let idx_b = self.heap_to_list[pos_b];
		self.heap_to_list.swap(pos_a, pos_b);
		self.list_to_heap.swap(idx_a, idx_b);
	}

	/// Fix the position of a single heap entry when its value changes.
	fn heap_fixup(&mut self, mut pos: usize, up_only: bool) {
		if pos >= self.heap_sorted {
			// ignore if the entry is outside the current valid heap
			return;
		}

		let val = self.heap_value(pos);

		// shift up an entry that is less than its parent
		while pos > 0 {
			let parent_pos = (pos - 1) / 2;
			let parent_val = self.heap_value(parent_pos);
			if val < parent_val {
				self.heap_swap(pos, parent_pos);
				pos = parent_pos;
			} else {
				break;
			}
		}

		// shift down an entry that is greater than either children
		if !up_only {
			self.heap_shift_down(pos, val, self.heap_sorted);
		}
	}

	/// Fix the position of a single heap entry by shifting it down in case it
	/// is greater than either of its children.
	///
	/// This assumes that both children sub-trees hold the heap invariant.
	fn heap_shift_down(&mut self, mut pos: usize, val: T::Precedence, heap_len: usize) {
		loop {
			let lhs = pos * 2 + 1;
			let rhs = pos * 2 + 2;
			if lhs >= heap_len {
				break;
			}

			let lhs_val = self.heap_value(lhs);
			let (child_pos, child_val) = if rhs < heap_len {
				let rhs_val = self.heap_value(rhs);
				if rhs_val < lhs_val {
					(rhs, rhs_val)
				} else {
					(lhs, lhs_val)
				}
			} else {
				(lhs, lhs_val)
			};
			if child_val < val {
				self.heap_swap(pos, child_pos);
				pos = child_pos;
			} else {
				break;
			}
		}
	}
}

struct ValueEntry<'a, T: IsNode> {
	value: T::Val,
	nodes: BoundNodes<'a, T>,
}

pub struct ScopeValueIterator<'a, 'b, T: IsNode> {
	parent: &'b ValueTable<'a, T>,
	next: usize,
	last: usize,
}

impl<'a, 'b, T: IsNode> Iterator for ScopeValueIterator<'a, 'b, T> {
	type Item = (&'b T::Val, &'b BoundNodes<'a, T>);

	fn next(&mut self) -> Option<Self::Item> {
		if self.next < self.last {
			let index = self.parent.heap_to_list[self.next];
			let item = &self.parent.list[index];
			let output = (&item.value, &item.nodes);
			self.next += 1;
			Some(output)
		} else {
			None
		}
	}
}

//====================================================================================================================//
// ScopeTree
//====================================================================================================================//

/// Maps scope values for a specific key in a [`ScopeMap`].
struct ScopeTree<'a, T: IsNode> {
	root_index: usize,
	root_value: bool,
	list: Vec<BindSegment<'a, T>>,
}

impl<'a, T: IsNode> ScopeTree<'a, T> {
	pub fn new(values: &mut ValueTable<'a, T>) -> Self {
		Self {
			root_index: values.new_entry(T::Val::default(), BoundNodes::new()),
			root_value: false,
			list: Default::default(),
		}
	}

	pub fn bind(&mut self, values: &mut ValueTable<'a, T>, scope: Scope, value: T::Val) {
		match scope {
			Scope::Root => {
				self.root_value = true;
				values.set_value(self.root_index, value);
			}
			Scope::Range(scope_sta, scope_end) => {
				let length = self.list.len();

				// find the first index that could overlap the scope
				let mut index = self.list.partition_point(|x| x.end < scope_sta);

				if index >= length || self.list[index].sta > scope_end {
					// no overlap, just insert a new segment
					let sta = scope_sta;
					let end = scope_end;
					let nodes = values.extract_range(self.root_index, sta, end);
					self.list.push(BindSegment {
						scope_sta,
						scope_end,
						sta,
						end,
						index: values.new_entry(value, nodes),
						_data: Default::default(),
					});
				} else {
					// check if the scope prefix is unbound
					let next = &self.list[index];
					if scope_sta < next.sta {
						let sta = scope_sta;
						let end = next.sta - 1;
						let nodes = values.extract_range(self.root_index, sta, end);
						self.list.push(BindSegment {
							scope_sta,
							scope_end,
							sta,
							end,
							index: values.new_entry(value.clone(), nodes),
							_data: Default::default(),
						});
						index += 1;
					}

					// overwrite existing segments
					let mut prev = None;
					while index < length && self.list[index].sta <= scope_end {
						let mut item = &mut self.list[index];
						index += 1;

						// fill any gaps between segments
						if let Some(prev) = prev {
							if (item.sta - prev) > 1 {
								let sta = prev + 1;
								let end = item.sta - 1;
								let nodes = values.extract_range(self.root_index, sta, end);
								self.list.push(BindSegment {
									scope_sta,
									scope_end,
									sta,
									end,
									index: values.new_entry(value.clone(), nodes),
									_data: Default::default(),
								});
								item = &mut self.list[index - 1]; // re-borrow after changing list
							}
						}
						prev = Some(item.end);

						// don't touch a segment from a more specific scope
						if !item.can_bind(scope_sta, scope_end) {
							continue;
						}

						if scope_sta > item.sta {
							// split the first segment
							let sta = scope_sta;
							let end = item.end;
							let nodes = values.extract_range(item.index, sta, end);
							item.end = sta - 1;
							self.list.push(BindSegment {
								scope_sta,
								scope_end,
								sta,
								end,
								index: values.new_entry(value.clone(), nodes),
								_data: Default::default(),
							});
						} else if scope_end < item.end {
							// split the last segment
							let sta = item.sta;
							let end = scope_end;
							let nodes = values.extract_range(item.index, sta, end);
							item.sta = end + 1;
							self.list.push(BindSegment {
								scope_sta,
								scope_end,
								sta,
								end: scope_end,
								index: values.new_entry(value.clone(), nodes),
								_data: Default::default(),
							});
						} else {
							// segment is fully contained in the new binding, so just overwrite it
							item.scope_sta = scope_sta;
							item.scope_end = scope_end;
							values.set_value(item.index, value.clone());
						}
					}
				}

				self.list.sort_by_key(|x| x.sta);
			}
		}
	}

	pub fn get<'b>(&self, values: &'b ValueTable<'a, T>, offset: usize) -> Option<&'b T::Val> {
		let index = self.list.partition_point(|x| x.end < offset);
		if let Some(item) = self.list.get(index) {
			if offset >= item.sta && offset <= item.end {
				Some(values.get_value(item.index))
			} else {
				if self.root_value {
					Some(values.get_value(self.root_index))
				} else {
					None
				}
			}
		} else {
			if self.root_value {
				Some(values.get_value(self.root_index))
			} else {
				None
			}
		}
	}

	pub fn add_node(&mut self, values: &mut ValueTable<'a, T>, node: Node<'a, T>) {
		if let Some(segment) = self.find_segment_mut(node.offset()) {
			values.add_node(segment.index, node)
		} else {
			values.add_node(self.root_index, node)
		}
	}

	pub fn remove_node(&mut self, values: &mut ValueTable<'a, T>, node: &Node<'a, T>) {
		if let Some(segment) = self.find_segment_mut(node.offset()) {
			values.remove_node(segment.index, node)
		} else {
			values.remove_node(self.root_index, node)
		}
	}

	fn find_segment_mut(&mut self, offset: usize) -> Option<&mut BindSegment<'a, T>> {
		use std::cmp::Ordering;
		if let Ok(index) = self.list.binary_search_by(|it| {
			let (sta, end) = (it.sta, it.end);
			if offset >= sta && offset <= end {
				Ordering::Equal
			} else {
				if offset < sta {
					Ordering::Less
				} else {
					Ordering::Greater
				}
			}
		}) {
			Some(&mut self.list[index])
		} else {
			None
		}
	}
}

//----------------------------------------------------------------------------------------------------------------//
// BindSegment
//----------------------------------------------------------------------------------------------------------------//

struct BindSegment<'a, T: IsNode> {
	scope_sta: usize,
	scope_end: usize,
	index: usize,
	sta: usize,
	end: usize,
	_data: PhantomData<ValueEntry<'a, T>>,
}

impl<'a, T: IsNode> BindSegment<'a, T> {
	/// Check if the given new scope binds to the current segment. The given
	/// scope should intersect the current segment.
	///
	/// A bind only overwrites a segment from an equal or less specific scope.
	pub fn can_bind(&self, new_sta: usize, new_end: usize) -> bool {
		let cur_sta = self.scope_sta;
		let cur_end = self.scope_end;

		// don't allow partially overlapping scopes
		let partial = cur_sta > new_sta && new_end < cur_end || new_sta > cur_sta && cur_end < new_end;
		if partial {
			let (a, b) = (cur_sta, cur_end);
			let (c, d) = (new_sta, new_end);
			panic!("partially overlapping scopes are not allowed: {a}-{b} with {c}-{d}");
		}

		let can_bind = new_sta >= cur_sta && new_end <= cur_end;
		can_bind
	}
}

//====================================================================================================================//
// BoundNodes
//====================================================================================================================//

pub struct BoundNodes<'a, T: IsNode> {
	id: usize,
	nodes: Vec<Node<'a, T>>,
	sorted: usize,
	removed: bool,
}

impl<'a, T: IsNode> BoundNodes<'a, T> {
	pub fn new() -> Self {
		Self {
			id: new_id(),
			nodes: Default::default(),
			sorted: 0,
			removed: false,
		}
	}

	pub fn add_node(&mut self, node: Node<'a, T>) {
		let length = self.nodes.len();
		let sorted = self.sorted == length
			&& if length == 0 {
				true
			} else {
				self.nodes[length - 1].offset() < node.offset()
			};
		node.replace_binding(0, self.id)
			.expect("BoundNodes: adding node already bound");

		self.nodes.push(node);
		if sorted {
			self.sorted = self.nodes.len();
		}
	}

	pub fn remove_node(&mut self, node: &Node<'a, T>) {
		node.replace_binding(self.id, 0)
			.expect("BoundNodes: removing node not on the list");
		self.removed = true;
	}

	pub fn extract_range(&mut self, sta: usize, end: usize) -> Self {
		self.fix_nodes();
		let head = self.nodes.partition_point(|x| x.offset() < sta);
		let tail = &self.nodes[head..];
		let len = tail.partition_point(|x| x.offset() <= end);
		let nodes = self.nodes.drain(head..head + len).collect();
		let output = Self {
			id: new_id(),
			nodes,
			sorted: len,
			removed: false,
		};
		for it in output.nodes.iter() {
			it.replace_binding(self.id, output.id)
				.expect("BoundNodes: extracted node was moved in the meantime");
		}
		output
	}

	pub fn list(&mut self) -> &[Node<'a, T>] {
		self.fix_nodes();
		&self.nodes
	}

	fn fix_nodes(&mut self) {
		if self.removed {
			self.nodes.retain(|x| x.binding() != self.id);
			self.removed = false;
		}
		if self.sorted < self.nodes.len() {
			self.nodes.sort_by_key(|x| x.offset());
			self.sorted = self.nodes.len();
		}
	}
}

impl<'a, T: IsNode> Debug for BoundNodes<'a, T> {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{:?}", self.nodes)
	}
}

//====================================================================================================================//
// Tests
//====================================================================================================================//

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	pub fn basic_root_binding() {
		let mut map = ScopeMap::<Bind>::new();
		map.bind("a", Scope::Root, 1);
		map.bind("b", Scope::Root, 2);

		assert_eq!(map.get(&"a", 0), Some(&1));
		assert_eq!(map.get(&"a", 1), Some(&1));
		assert_eq!(map.get(&"b", 0), Some(&2));
		assert_eq!(map.get(&"b", 9), Some(&2));

		assert_eq!(map.get(&"c", 0), None);
	}

	#[test]
	pub fn binding_offset() {
		let mut map = ScopeMap::<Bind>::new();
		map.bind("a", Scope::Root, 10);
		map.bind("a", Scope::Range(1, 2), 11);
		map.bind("a", Scope::Range(3, 4), 12);
		map.bind("a", Scope::Range(4, 4), 13);
		map.bind("a", Scope::Range(6, 7), 14);
		map.bind("a", Scope::Range(3, 7), 15);
		map.bind("b", Scope::Range(1, 3), 20);

		assert_eq!(map.get(&"a", 0), Some(&10)); // root
		assert_eq!(map.get(&"a", 1), Some(&11));
		assert_eq!(map.get(&"a", 2), Some(&11));
		assert_eq!(map.get(&"a", 3), Some(&12));
		assert_eq!(map.get(&"a", 4), Some(&13));
		assert_eq!(map.get(&"a", 5), Some(&15)); // 3-7
		assert_eq!(map.get(&"a", 6), Some(&14));
		assert_eq!(map.get(&"a", 7), Some(&14));
		assert_eq!(map.get(&"a", 8), Some(&10)); // root
		assert_eq!(map.get(&"a", 9), Some(&10)); // root

		assert_eq!(map.get(&"b", 1), Some(&20));
		assert_eq!(map.get(&"b", 2), Some(&20));
		assert_eq!(map.get(&"b", 3), Some(&20));
		assert_eq!(map.get(&"b", 0), None);
	}

	#[test]
	pub fn basic_precedence() {
		let mut map = ScopeMap::<Bind>::new();
		map.bind("d", Scope::Root, 4);
		map.bind("a", Scope::Root, 1);
		map.bind("c", Scope::Root, 3);
		map.bind("b", Scope::Root, 2);
		map.bind("g", Scope::Root, 7);

		let check = |map: &mut ScopeMap<Bind>, n: i32| {
			let actual = map.shift_next().unwrap().map(|(a, _)| *a).collect::<Vec<_>>();
			assert_eq!(actual, vec![n])
		};

		check(&mut map, 1);
		check(&mut map, 2);
		check(&mut map, 3);
		check(&mut map, 4);

		map.bind("f", Scope::Root, 6);
		map.bind("e", Scope::Root, 5);

		check(&mut map, 5);
		check(&mut map, 6);
		check(&mut map, 7);
	}

	#[derive(Copy, Clone)]
	struct Bind;

	impl IsNode for Bind {
		type Expr<'a> = BindExpr;
		type Key = &'static str;
		type Val = i32;
		type Precedence = i32;

		fn get_precedence(val: &Self::Val) -> Self::Precedence {
			*val
		}
	}

	#[derive(Debug)]
	struct BindExpr;

	impl<'a> IsExpr<'a, Bind> for BindExpr {}
}
