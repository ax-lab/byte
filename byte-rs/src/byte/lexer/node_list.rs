use std::{ops::*, sync::Arc};

use super::*;

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

	pub fn len(&self) -> usize {
		self.list.len()
	}

	pub fn get(&self, index: usize) -> Option<&Node> {
		self.list.get(index)
	}

	pub fn range<T: RangeBounds<usize>>(&self, range: T) -> NodeStream {
		let range = Str::compute_range(range, self.len());
		NodeStream::new(self.list.clone(), range.start, range.end)
	}

	pub fn iter(&self) -> impl Iterator<Item = &Node> {
		self.list.iter()
	}
}

//====================================================================================================================//
// Traits
//====================================================================================================================//

impl IntoIterator for NodeList {
	type Item = Node;

	type IntoIter = NodeStream;

	fn into_iter(self) -> Self::IntoIter {
		self.range(..)
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
