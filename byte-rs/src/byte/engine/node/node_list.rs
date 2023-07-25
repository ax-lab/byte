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
				if len <= NODE_FIX_LEN {
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

const NODE_FIX_LEN: usize = 3;

#[derive(Copy, Clone)]
struct NodeListFix<'a, T: IsNode> {
	len: usize,
	ptr: [*const NodeData<'a, T>; NODE_FIX_LEN],
}

#[derive(Copy, Clone)]
struct NodeListVec<'a, T: IsNode> {
	len: usize,
	ptr: *const *const NodeData<'a, T>,
}
