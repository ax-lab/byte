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
			values: ValueTable {
				list: Default::default(),
			},
		}
	}

	// TODO: change interface to operate with the nodes directly.

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

	pub fn add_node(&mut self, key: T::Key, node: Node<'a, T>) {
		let entry = self
			.table
			.entry(key)
			.or_insert_with(|| ScopeTree::new(&mut self.values));
		entry.add_node(&mut self.values, node)
	}
}

//----------------------------------------------------------------------------------------------------------------//
// ValueTable and Entry
//----------------------------------------------------------------------------------------------------------------//

/// Stores an entry for each value and node segment pair in the scope.
struct ValueTable<'a, T: IsNode> {
	list: Vec<ValueEntry<'a, T>>,
}

impl<'a, T: IsNode> ValueTable<'a, T> {
	pub fn new_entry(&mut self, value: T::Val, nodes: BoundNodes<'a, T>) -> usize {
		let index = self.list.len();
		self.list.push(ValueEntry { value, nodes });
		index
	}

	pub fn extract_range(&mut self, index: usize, sta: usize, end: usize) -> BoundNodes<'a, T> {
		self.list[index].nodes.extract_range(sta, end)
	}

	pub fn add_node(&mut self, index: usize, node: Node<'a, T>) {
		self.list[index].nodes.add_node(node);
	}

	pub fn get_value(&self, index: usize) -> &T::Val {
		&self.list[index].value
	}

	pub fn set_value(&mut self, index: usize, value: T::Val) {
		self.list[index].value = value;
	}
}

struct ValueEntry<'a, T: IsNode> {
	value: T::Val,
	nodes: BoundNodes<'a, T>,
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

struct BoundNodes<'a, T: IsNode> {
	nodes: Vec<Node<'a, T>>,
	sorted: usize,
}

impl<'a, T: IsNode> BoundNodes<'a, T> {
	pub fn new() -> Self {
		Self {
			nodes: Default::default(),
			sorted: 0,
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
		self.nodes.push(node);
		if sorted {
			self.sorted = self.nodes.len();
		}
	}

	pub fn extract_range(&mut self, sta: usize, end: usize) -> Self {
		self.ensure_sorted();
		let head = self.nodes.partition_point(|x| x.offset() < sta);
		let tail = &self.nodes[head..];
		let len = tail.partition_point(|x| x.offset() <= end);
		let nodes = self.nodes.drain(head..head + len).collect();
		Self { nodes, sorted: len }
	}

	pub fn ensure_sorted(&mut self) {
		if self.sorted < self.nodes.len() {
			self.nodes.sort_by_key(|x| x.offset());
			self.sorted = self.nodes.len();
		}
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
