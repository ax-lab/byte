use super::*;

/// Maintains a list of [`NodeValue`] for evaluation.
#[derive(Default, Clone, Eq, PartialEq)]
pub struct NodeValueList {
	nodes: Arc<Vec<NodeValue>>,
}

has_traits!(NodeValueList: IsNode);

impl IsNode for NodeValueList {}

impl NodeValueList {
	pub fn new<T: IntoIterator<Item = NodeValue>>(items: T) -> Self {
		let nodes: Vec<NodeValue> = items.into_iter().collect();
		Self { nodes: nodes.into() }
	}

	pub fn empty() -> Self {
		Self {
			nodes: Default::default(),
		}
	}

	pub fn single(node: NodeValue) -> Self {
		NodeValueList {
			nodes: Arc::new(vec![node]),
		}
	}

	pub fn is_empty(&self) -> bool {
		self.len() == 0
	}

	pub fn len(&self) -> usize {
		self.nodes.len()
	}

	pub fn range<T: RangeBounds<usize>>(&self, range: T) -> NodeValueList {
		let range = compute_range(range, self.len());
		let range = &self.nodes[range];
		if range.len() == self.len() {
			self.clone()
		} else {
			NodeValueList {
				nodes: range.to_vec().into(),
			}
		}
	}

	pub fn slice<T: RangeBounds<usize>>(&self, range: T) -> &[NodeValue] {
		let range = compute_range(range, self.len());
		&self.nodes[range]
	}

	pub fn iter(&self) -> NodeValueListIterator {
		NodeValueListIterator { list: self, index: 0 }
	}

	pub fn as_slice(&self) -> &[NodeValue] {
		self.nodes.as_slice()
	}

	//----------------------------------------------------------------------------------------------------------------//
	// Helper methods
	//----------------------------------------------------------------------------------------------------------------//

	pub fn split_by<F: Fn(&NodeValue) -> bool>(&self, predicate: F) -> Vec<NodeValueList> {
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

impl Index<usize> for NodeValueList {
	type Output = NodeValue;

	fn index(&self, index: usize) -> &Self::Output {
		&self.nodes[index]
	}
}

/// Iterator for a [`NodeValueList`].
pub struct NodeValueListIterator<'a> {
	list: &'a NodeValueList,
	index: usize,
}

impl<'a> Iterator for NodeValueListIterator<'a> {
	type Item = &'a NodeValue;

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

impl Debug for NodeValueList {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		write!(f, "<NodeValueList")?;
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
