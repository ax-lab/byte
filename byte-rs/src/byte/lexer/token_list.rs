use std::{ops::*, sync::Arc};

use super::*;

#[derive(Clone)]
pub struct TokenList {
	errors: Errors,
	list: Arc<Vec<Node>>,
}

impl TokenList {
	/// Parses the given input using the lexer.
	pub fn parse(input: Input, scanner: &mut Scanner) -> Self {
		let mut cursor = input.cursor();
		let mut errors = Errors::new();

		let mut list = Vec::new();
		while let Some(node) = scanner.read(&mut cursor, &mut errors) {
			// TODO: parse lexer directives here
			list.push(node);
			if errors.len() > 0 {
				break;
			}
		}

		let list = list.into();
		Self { errors, list }
	}

	pub fn has_errors(&self) -> bool {
		self.errors.len() > 0
	}

	pub fn errors(&self) -> &Errors {
		&self.errors
	}

	pub fn len(&self) -> usize {
		self.list.len()
	}

	pub fn get(&self, index: usize) -> Option<&Node> {
		self.list.get(index)
	}

	pub fn range<T: RangeBounds<usize>>(&self, range: T) -> TokenStream {
		let range = Str::compute_range(range, self.len());
		TokenStream::new(self.list.clone(), range.start, range.end)
	}

	pub fn iter(&self) -> impl Iterator<Item = &Node> {
		self.list.iter()
	}
}

//====================================================================================================================//
// Traits
//====================================================================================================================//

impl IntoIterator for TokenList {
	type Item = Node;

	type IntoIter = TokenStream;

	fn into_iter(self) -> Self::IntoIter {
		self.range(..)
	}
}

impl Index<usize> for TokenList {
	type Output = Node;

	fn index(&self, index: usize) -> &Self::Output {
		&self.list[index]
	}
}

impl Index<Range<usize>> for TokenList {
	type Output = [Node];

	fn index(&self, index: Range<usize>) -> &Self::Output {
		&self.list[index]
	}
}

impl Index<RangeInclusive<usize>> for TokenList {
	type Output = [Node];

	fn index(&self, index: RangeInclusive<usize>) -> &Self::Output {
		&self.list[index]
	}
}

impl Index<RangeFrom<usize>> for TokenList {
	type Output = [Node];

	fn index(&self, index: RangeFrom<usize>) -> &Self::Output {
		&self.list[index]
	}
}

impl Index<RangeTo<usize>> for TokenList {
	type Output = [Node];

	fn index(&self, index: RangeTo<usize>) -> &Self::Output {
		&self.list[index]
	}
}

impl Index<RangeToInclusive<usize>> for TokenList {
	type Output = [Node];

	fn index(&self, index: RangeToInclusive<usize>) -> &Self::Output {
		&self.list[index]
	}
}

impl Index<RangeFull> for TokenList {
	type Output = [Node];

	fn index(&self, index: RangeFull) -> &Self::Output {
		&self.list[index]
	}
}
