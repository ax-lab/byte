use std::{ops::*, sync::Arc};

use super::*;

//====================================================================================================================//
// NodeList
//====================================================================================================================//

#[derive(Clone)]
pub struct NodeList {
	list: Arc<Vec<Node>>,
}

impl NodeList {
	/// Parses the given input using the given [`Scanner`].
	pub fn tokenize(input: Input, scanner: &mut Scanner, errors: &mut Errors) -> Self {
		let mut cursor = input.cursor();

		let mut list = Vec::new();
		while let Some(node) = scanner.read(&mut cursor, errors) {
			// TODO: parse lexer directives here
			list.push(node);
			if errors.len() > 0 {
				break;
			}
		}

		let list = list.into();
		Self { list }
	}

	/// Creates a new [`NodeList`] from an iterator of [`Node`].
	pub fn new<T: IntoIterator<Item = Node>>(input: T) -> Self {
		let list = Vec::from_iter(input).into();
		Self { list }
	}

	/// Number of nodes in the list.
	pub fn len(&self) -> usize {
		self.list.len()
	}

	/// Get a node by its index.
	pub fn get(&self, index: usize) -> Option<&Node> {
		self.list.get(index)
	}

	/// Return a range of the list.
	pub fn range<T: RangeBounds<usize>>(&self, range: T) -> NodeRange {
		let range = Str::compute_range(range, self.len());
		NodeRange {
			source: self.list.clone(),
			start: range.start,
			end: range.end,
		}
	}

	/// Returns an iterator over the nodes in the list.
	pub fn iter(&self) -> impl Iterator<Item = &Node> {
		self.list.iter()
	}
}

//====================================================================================================================//
// NodeRange
//====================================================================================================================//

/// Range of [`Node`] from a [`NodeList`].
#[derive(Clone)]
pub struct NodeRange {
	source: Arc<Vec<Node>>,
	start: usize,
	end: usize,
}

impl NodeRange {
	/// Number of nodes in the range.
	pub fn len(&self) -> usize {
		self.as_slice().len()
	}

	/// Get a node by its index in the range.
	pub fn get(&self, index: usize) -> Option<&Node> {
		self.as_slice().get(index)
	}

	/// Returns a sub-range of this range.
	pub fn sub_range<T: RangeBounds<usize>>(&self, range: T) -> NodeRange {
		let range = Str::compute_range(range, self.len());
		NodeRange {
			source: self.source.clone(),
			start: self.start + range.start,
			end: self.end + range.end,
		}
	}

	/// Returns an iterator over the nodes in the range.
	pub fn iter(&self) -> impl Iterator<Item = &Node> {
		self.as_slice().iter()
	}

	pub fn as_slice(&self) -> &[Node] {
		&self.source[self.start..self.end]
	}
}

//--------------------------------------------------------------------------------------------------------------------//
// NodeList traits
//--------------------------------------------------------------------------------------------------------------------//

impl IntoIterator for NodeList {
	type Item = Node;

	type IntoIter = NodeStream;

	fn into_iter(self) -> Self::IntoIter {
		self.range(..).into_iter()
	}
}

impl Index<usize> for NodeList {
	type Output = Node;

	fn index(&self, index: usize) -> &Self::Output {
		&self.list[index]
	}
}

impl Index<Range<usize>> for NodeList {
	type Output = [Node];

	fn index(&self, index: Range<usize>) -> &Self::Output {
		&self.list[index]
	}
}

impl Index<RangeInclusive<usize>> for NodeList {
	type Output = [Node];

	fn index(&self, index: RangeInclusive<usize>) -> &Self::Output {
		&self.list[index]
	}
}

impl Index<RangeFrom<usize>> for NodeList {
	type Output = [Node];

	fn index(&self, index: RangeFrom<usize>) -> &Self::Output {
		&self.list[index]
	}
}

impl Index<RangeTo<usize>> for NodeList {
	type Output = [Node];

	fn index(&self, index: RangeTo<usize>) -> &Self::Output {
		&self.list[index]
	}
}

impl Index<RangeToInclusive<usize>> for NodeList {
	type Output = [Node];

	fn index(&self, index: RangeToInclusive<usize>) -> &Self::Output {
		&self.list[index]
	}
}

impl Index<RangeFull> for NodeList {
	type Output = [Node];

	fn index(&self, index: RangeFull) -> &Self::Output {
		&self.list[index]
	}
}

//--------------------------------------------------------------------------------------------------------------------//
// NodeRange traits
//--------------------------------------------------------------------------------------------------------------------//

impl IntoIterator for NodeRange {
	type Item = Node;

	type IntoIter = NodeStream;

	fn into_iter(self) -> Self::IntoIter {
		NodeStream::new(self)
	}
}

impl Index<usize> for NodeRange {
	type Output = Node;

	fn index(&self, index: usize) -> &Self::Output {
		&self.as_slice()[index]
	}
}

impl Index<Range<usize>> for NodeRange {
	type Output = [Node];

	fn index(&self, index: Range<usize>) -> &Self::Output {
		&self.as_slice()[index]
	}
}

impl Index<RangeInclusive<usize>> for NodeRange {
	type Output = [Node];

	fn index(&self, index: RangeInclusive<usize>) -> &Self::Output {
		&self.as_slice()[index]
	}
}

impl Index<RangeFrom<usize>> for NodeRange {
	type Output = [Node];

	fn index(&self, index: RangeFrom<usize>) -> &Self::Output {
		&self.as_slice()[index]
	}
}

impl Index<RangeTo<usize>> for NodeRange {
	type Output = [Node];

	fn index(&self, index: RangeTo<usize>) -> &Self::Output {
		&self.as_slice()[index]
	}
}

impl Index<RangeToInclusive<usize>> for NodeRange {
	type Output = [Node];

	fn index(&self, index: RangeToInclusive<usize>) -> &Self::Output {
		&self.as_slice()[index]
	}
}

impl Index<RangeFull> for NodeRange {
	type Output = [Node];

	fn index(&self, index: RangeFull) -> &Self::Output {
		&self.as_slice()[index]
	}
}
