use super::*;

/// Iterator over a [`NodeList`].
pub struct NodeIterator<'a, T: IsNode> {
	list: NodeList<'a, T>,
	next: usize,
}

impl<'a, T: IsNode> NodeIterator<'a, T> {
	pub fn empty() -> Self {
		NodeList::empty().into_iter()
	}

	pub fn single(node: &Node<'a, T>) -> Self {
		NodeList::single(*node).into_iter()
	}

	pub fn len(&self) -> usize {
		self.list.len()
	}

	pub fn to_list(&self) -> NodeList<'a, T> {
		self.list
	}
}

impl<'a, T: IsNode> Iterator for NodeIterator<'a, T> {
	type Item = Node<'a, T>;

	fn next(&mut self) -> Option<Self::Item> {
		let next = self.list.get(self.next);
		if next.is_some() {
			self.next += 1;
		}
		next
	}
}

impl<'a, T: IsNode> IntoIterator for NodeList<'a, T> {
	type Item = Node<'a, T>;
	type IntoIter = NodeIterator<'a, T>;

	fn into_iter(self) -> Self::IntoIter {
		NodeIterator { list: self, next: 0 }
	}
}
