use std::collections::HashMap;

use super::*;

// TODO: remove nodes from a scope tree

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Scope {
	Root,
	Range(usize, usize),
}

impl Scope {
	pub fn contains(&self, other: &Scope) -> bool {
		match self {
			Scope::Root => true,
			Scope::Range(a0, a1) => match other {
				Scope::Root => false,
				Scope::Range(b0, b1) => {
					let overlap = a0 <= b1 && b0 <= a1;
					let contain = b0 >= a1 && b1 <= a1;
					if overlap && contain {
						panic!("intersecting scopes are not allowed");
					}
					contain
				}
			},
		}
	}
}

impl PartialOrd for Scope {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl Ord for Scope {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		use std::cmp::Ordering;
		match self {
			Scope::Root => match other {
				Scope::Root => Ordering::Equal,
				Scope::Range(..) => Ordering::Less,
			},
			Scope::Range(a0, a1) => match other {
				Scope::Root => Ordering::Greater,
				Scope::Range(b0, b1) => a0.cmp(b0).then(a1.cmp(b1)),
			},
		}
	}
}

pub struct BindingMap<'a, T: IsNode> {
	table: HashMap<T::Key, ScopeTree<'a, T>>,
}

impl<'a, T: IsNode> BindingMap<'a, T> {
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
	pub fn bind(&mut self, key: T::Key, scope: Scope, data: T::Val) {
		let entry = self.table.entry(key).or_insert_with(|| ScopeTree::new());
		entry.bind(scope, data)
	}

	pub fn get(&self, key: &T::Key, offset: usize) -> Option<&T::Val> {
		if let Some(entry) = self.table.get(key) {
			entry.get(offset)
		} else {
			None
		}
	}

	pub fn add_node(&mut self, key: T::Key, node: Node<'a, T>) {
		let entry = self.table.entry(key).or_insert_with(|| ScopeTree::new());
		entry.add_node(node)
	}
}

/// A scope tree for a specific key in a [`BindingMap`].
struct ScopeTree<'a, T: IsNode> {
	root: Option<T::Val>,
	root_nodes: BoundNodes<'a, T>,
	list: Vec<BindSegment<'a, T>>,
}

impl<'a, T: IsNode> ScopeTree<'a, T> {
	pub fn new() -> Self {
		Self {
			root: None,
			root_nodes: BoundNodes::new(),
			list: Default::default(),
		}
	}

	pub fn bind(&mut self, scope: Scope, value: T::Val) {
		match scope {
			Scope::Root => self.root = Some(value),
			Scope::Range(scope_sta, scope_end) => {
				let length = self.list.len();

				// find the first index that could overlap the scope
				let mut index = self.list.partition_point(|x| x.end < scope_sta);

				if index >= length || self.list[index].sta > scope_end {
					// no overlap, just insert a new segment
					let sta = scope_sta;
					let end = scope_end;
					let nodes = self.root_nodes.extract_range(sta, end);
					self.list.push(BindSegment {
						scope_sta,
						scope_end,
						nodes,
						sta,
						end,
						val: value,
					});
				} else {
					// check if the scope prefix is unbound
					let next = &self.list[index];
					if scope_sta < next.sta {
						let sta = scope_sta;
						let end = next.sta - 1;
						let nodes = self.root_nodes.extract_range(sta, end);
						self.list.push(BindSegment {
							scope_sta,
							scope_end,
							nodes,
							sta,
							end,
							val: value.clone(),
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
								let nodes = self.root_nodes.extract_range(sta, end);
								self.list.push(BindSegment {
									scope_sta,
									scope_end,
									nodes,
									sta,
									end,
									val: value.clone(),
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
							let nodes = item.nodes.extract_range(sta, end);
							item.end = sta - 1;
							self.list.push(BindSegment {
								scope_sta,
								scope_end,
								nodes,
								sta,
								end,
								val: value.clone(),
							});
						} else if scope_end < item.end {
							// split the last segment
							let sta = item.sta;
							let end = scope_end;
							let nodes = item.nodes.extract_range(sta, end);
							item.sta = end + 1;
							self.list.push(BindSegment {
								scope_sta,
								scope_end,
								nodes,
								sta,
								end: scope_end,
								val: value.clone(),
							});
						} else {
							// segment is fully contained in the new binding, so just overwrite it
							item.scope_sta = scope_sta;
							item.scope_end = scope_end;
							item.val = value.clone();
						}
					}
				}

				self.list.sort_by_key(|x| x.sta);
			}
		}
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
	}

	pub fn get(&self, offset: usize) -> Option<&T::Val> {
		let index = self.list.partition_point(|x| x.end < offset);
		if let Some(item) = self.list.get(index) {
			if offset >= item.sta && offset <= item.end {
				Some(&item.val)
			} else {
				self.root.as_ref()
			}
		} else {
			self.root.as_ref()
		}
	}

	pub fn add_node(&mut self, node: Node<'a, T>) {
		if let Some(segment) = self.find_segment_mut(node.offset()) {
			segment.nodes.add_node(node)
		} else {
			self.root_nodes.add_node(node)
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

struct BindSegment<'a, T: IsNode> {
	scope_sta: usize,
	scope_end: usize,
	nodes: BoundNodes<'a, T>,
	sta: usize,
	end: usize,
	val: T::Val,
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
		let mut map = BindingMap::<Bind>::new();
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
		let mut map = BindingMap::<Bind>::new();
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
