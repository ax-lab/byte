use super::*;

/// Maintains a list of [`Node`] for evaluation.
#[derive(Default, Clone, Eq, PartialEq)]
pub struct NodeList {
	nodes: Arc<Vec<Node>>,
}

has_traits!(NodeList: IsNode);

impl IsNode for NodeList {}

impl NodeList {
	pub fn new<T: IntoIterator<Item = Node>>(items: T) -> Self {
		let nodes: Vec<Node> = items.into_iter().collect();
		Self { nodes: nodes.into() }
	}

	pub fn empty() -> Self {
		Self {
			nodes: Default::default(),
		}
	}

	pub fn single(node: Node) -> Self {
		NodeList {
			nodes: Arc::new(vec![node]),
		}
	}

	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}

	pub fn len(&self) -> usize {
		self.nodes.len()
	}

	pub fn range<T: RangeBounds<usize>>(&self, range: T) -> NodeList {
		let range = compute_range(range, self.len());
		let range = &self.nodes[range];
		if range.len() == self.len() {
			self.clone()
		} else {
			NodeList {
				nodes: range.to_vec().into(),
			}
		}
	}

	pub fn slice<T: RangeBounds<usize>>(&self, range: T) -> &[Node] {
		let range = compute_range(range, self.len());
		&self.nodes[range]
	}

	pub fn iter(&self) -> NodeListIterator {
		NodeListIterator { list: self, index: 0 }
	}

	pub fn as_slice(&self) -> &[Node] {
		self.nodes.as_slice()
	}

	//----------------------------------------------------------------------------------------------------------------//
	// Helper methods
	//----------------------------------------------------------------------------------------------------------------//

	pub fn split_by<F: Fn(&Node) -> bool>(&self, predicate: F) -> Vec<NodeList> {
		let mut output = Vec::new();
		let mut item = None;
		for it in self.nodes.iter() {
			let current_item = item.get_or_insert(Vec::new());
			if predicate(it) {
				output.push(item.take().unwrap());
			} else {
				current_item.push(it.clone());
			}
		}
		if let Some(item) = item {
			output.push(item);
		}
		output.into_iter().map(|x| Self { nodes: x.into() }).collect()
	}
}

impl Index<usize> for NodeList {
	type Output = Node;

	fn index(&self, index: usize) -> &Self::Output {
		&self.nodes[index]
	}
}

/// Iterator for a [`NodeList`].
pub struct NodeListIterator<'a> {
	list: &'a NodeList,
	index: usize,
}

impl<'a> Iterator for NodeListIterator<'a> {
	type Item = &'a Node;

	fn next(&mut self) -> Option<Self::Item> {
		let index = self.index;
		if index < self.list.len() {
			self.index += 1;
			Some(&self.list.nodes[index])
		} else {
			None
		}
	}

	fn size_hint(&self) -> (usize, Option<usize>) {
		let remaining = self.list.len() - self.index;
		(remaining, Some(remaining))
	}
}

impl Debug for NodeList {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		write!(f, "<NodeList")?;
		for (n, it) in self.iter().enumerate() {
			let mut f = f.indented();
			write!(f, "\n>>> [{n}]")?;
			write!(f, "    # {:?}", it.id())?;
			write!(f, "")?;
			write!(f.indented_with("... "), "\n{it:?}")?;
		}
		write!(f, "\n>")
	}
}
