use std::ops::RangeBounds;

use super::*;

/// List of [`Node`].
#[derive(Copy, Clone)]
pub union NodeList<'a, T: IsNode> {
	fix: NodeListFix<'a, T>,
	vec: NodeListVec<'a, T>,
}

impl<'a, T: IsNode> NodeList<'a, T> {
	pub const fn empty() -> Self {
		let null = std::ptr::null_mut();
		NodeList {
			fix: NodeListFix {
				len: 0,
				ptr: [null, null, null],
			},
		}
	}

	pub fn single(node: Node<'a, T>) -> Self {
		let node = node.data;
		let null = std::ptr::null_mut();
		NodeList {
			fix: NodeListFix {
				len: 1,
				ptr: [node, null, null],
			},
		}
	}

	pub fn pair(a: Node<'a, T>, b: Node<'a, T>) -> Self {
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

	pub fn triple(a: Node<'a, T>, b: Node<'a, T>, c: Node<'a, T>) -> Self {
		let a = a.data;
		let b = b.data;
		let c = c.data;
		NodeList {
			fix: NodeListFix { len: 3, ptr: [a, b, c] },
		}
	}

	pub(crate) fn from_list(store: &NodeStore<T>, nodes: &[Node<'a, T>]) -> Self {
		let bytes = std::mem::size_of::<*const NodeData<'a, T>>() * nodes.len();
		let ptr = store.buffer.alloc(bytes) as *mut *const NodeData<'a, T>;
		let mut cur = ptr;
		for it in nodes.iter() {
			unsafe {
				cur.write(it.data);
				cur = cur.add(1);
			}
		}
		let vec = NodeListVec { len: nodes.len(), ptr };
		let _ = vec.len; // otherwise unused
		NodeList { vec }
	}

	#[inline(always)]
	pub fn len(&self) -> usize {
		unsafe { self.fix.len }
	}

	#[inline(always)]
	pub fn get(&self, index: usize) -> Option<Node<'a, T>> {
		let len = self.len();
		if index < len {
			let ptr = unsafe {
				if len <= NODE_LIST_FIX_LEN {
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

	pub fn iter(&self) -> NodeIterator<'a, T> {
		self.into_iter()
	}

	pub fn slice<R: RangeBounds<usize>>(&self, range: R) -> NodeList<'a, T> {
		let null = std::ptr::null_mut();
		let len = self.len();
		let range = crate::compute_range(range, len);
		assert!(range.end <= len && range.start <= range.end);
		if self.len() <= NODE_LIST_FIX_LEN {
			let list = unsafe { &self.fix.ptr[range] };
			let fix = match list.len() {
				0 => NodeListFix {
					len: 0,
					ptr: [null, null, null],
				},
				1 => NodeListFix {
					len: 1,
					ptr: [list[0], null, null],
				},
				2 => NodeListFix {
					len: 2,
					ptr: [list[0], list[1], null],
				},
				3 => NodeListFix {
					len: 3,
					ptr: [list[0], list[1], list[2]],
				},
				_ => unreachable!(),
			};
			NodeList { fix }
		} else {
			let ptr = unsafe { self.vec.ptr.add(range.start) };
			let len = range.end - range.start;
			if len <= NODE_LIST_FIX_LEN {
				unsafe {
					NodeList {
						fix: NodeListFix {
							len,
							ptr: [
								if len < 1 { null } else { *ptr },
								if len < 2 { null } else { *ptr.add(1) },
								if len < 3 { null } else { *ptr.add(2) },
							],
						},
					}
				}
			} else {
				NodeList {
					vec: NodeListVec { len, ptr },
				}
			}
		}
	}
}

impl<'a, T: IsNode> Default for NodeList<'a, T> {
	fn default() -> Self {
		Self::empty()
	}
}

impl<'a, T: IsNode> PartialEq for NodeList<'a, T> {
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

impl<'a, T: IsNode> Eq for NodeList<'a, T> {}

unsafe impl<'a, T: IsNode> Send for NodeList<'a, T> {}
unsafe impl<'a, T: IsNode> Sync for NodeList<'a, T> {}

impl<'a, T: IsNode> Debug for NodeList<'a, T> {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		// TODO: implement proper
		write!(f, "[")?;
		for (n, it) in self.iter().enumerate() {
			write!(f, "{}", if n == 0 { " " } else { ", " })?;
			write!(f, "{it:?}")?;
		}
		write!(f, " ]")
	}
}

//====================================================================================================================//
// Internals
//====================================================================================================================//

const NODE_LIST_FIX_LEN: usize = 3;

#[derive(Copy, Clone)]
struct NodeListFix<'a, T: IsNode> {
	len: usize,
	ptr: [*const NodeData<'a, T>; NODE_LIST_FIX_LEN],
}

#[derive(Copy, Clone)]
struct NodeListVec<'a, T: IsNode> {
	len: usize,
	ptr: *const *const NodeData<'a, T>,
}

//====================================================================================================================//
// Tests
//====================================================================================================================//

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn basic_list_and_slices() {
		let store = NodeStore::<List>::new();
		let mut nodes = store.new_node_set();
		let x0 = nodes.new_node(Expr::X(0));
		let x1 = nodes.new_node(Expr::X(1));
		let x2 = nodes.new_node(Expr::X(2));
		let x3 = nodes.new_node(Expr::X(3));
		let x4 = nodes.new_node(Expr::X(4));

		// test basic list creation
		let list0 = NodeList::<List>::empty();
		let list1 = NodeList::single(x0);
		let list2 = NodeList::pair(x0, x1);
		let list3 = NodeList::triple(x0, x1, x2);
		let list4 = nodes.list_from(&[x0, x1, x2, x3]);
		let list5 = nodes.list_from(&[x0, x1, x2, x3, x4]);

		let check = |list: &NodeList<_>, len: usize| {
			assert_eq!(list.len(), len);
			for i in 0..list.len() {
				assert_eq!(list.get(i).unwrap().expr(), &Expr::X(i));
			}
		};

		check(&list0, 0);
		check(&list1, 1);
		check(&list2, 2);
		check(&list3, 3);
		check(&list4, 4);
		check(&list5, 5);

		// test basic list creation with `list_from`
		let list0 = nodes.list_from(&[]);
		let list1 = nodes.list_from(&[x0]);
		let list2 = nodes.list_from(&[x0, x1]);
		let list3 = nodes.list_from(&[x0, x1, x2]);

		check(&list0, 0);
		check(&list1, 1);
		check(&list2, 2);
		check(&list3, 3);

		// test slicing a fixed length list
		let main = NodeList::triple(x0, x1, x2);
		let list0 = main.slice(0..0);
		let list1 = main.slice(0..1);
		let list2 = main.slice(0..2);
		let list3 = main.slice(0..3);
		check(&list0, 0);
		check(&list1, 1);
		check(&list2, 2);
		check(&list3, 3);

		// test slicing an allocated list
		let main = nodes.list_from(&[x0, x1, x2, x3, x4]);
		let list0 = main.slice(..0);
		let list1 = main.slice(..1);
		let list2 = main.slice(0..2);
		let list3 = main.slice(0..=2);
		let list4 = main.slice(..4);
		let list5 = main.slice(..);

		check(&list0, 0);
		check(&list1, 1);
		check(&list2, 2);
		check(&list3, 3);
		check(&list4, 4);
		check(&list5, 5);

		// test non-zero slicing a fixed length list
		let main = NodeList::triple(x3, x0, x1);
		let list0 = main.slice(3..);
		let list1 = main.slice(1..2);
		let list2 = main.slice(1..);

		check(&list0, 0);
		check(&list1, 1);
		check(&list2, 2);

		// test non-zero slicing an allocated list
		let main = nodes.list_from(&[x3, x4, x0, x1, x2]);
		let list0 = main.slice(2..2);
		let list1 = main.slice(2..3);
		let list2 = main.slice(2..4);
		let list3 = main.slice(2..5);

		check(&list0, 0);
		check(&list1, 1);
		check(&list2, 2);
		check(&list3, 3);
	}

	#[derive(Copy, Clone)]
	struct List;

	#[derive(Debug, Eq, PartialEq)]
	enum Expr {
		X(usize),
	}

	impl IsNode for List {
		type Expr<'a> = Expr;

		type Key = ();
	}

	impl<'a> IsExpr<'a, List> for Expr {}
}
