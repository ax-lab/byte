use std::ops::*;

use super::*;

/// Provides a stream of [`Node`] with helper methods for parsing.
#[derive(Clone)]
pub struct NodeStream {
	range: NodeRange,
	index: usize,
}

impl NodeStream {
	pub fn new(range: NodeRange) -> Self {
		Self { range, index: 0 }
	}

	pub fn len(&self) -> usize {
		self.range.len() - self.index
	}

	pub fn peek(&self) -> Option<&Node> {
		self.lookahead(0)
	}

	pub fn lookahead(&self, n: usize) -> Option<&Node> {
		self.range.get(self.index + n)
	}

	pub fn read(&mut self) -> Option<Node> {
		self.peek().cloned().map(|x| {
			self.index += 1;
			x
		})
	}

	pub fn undo(&mut self) {
		assert!(self.index > 0);
		self.index -= 1;
	}

	pub fn skip(&mut self, count: usize) {
		let index = self.index + count;
		self.index = std::cmp::min(index, self.range.len())
	}

	pub fn span(&self) -> Option<Span> {
		self.range.span_at(self.index)
	}

	pub fn pos(&self) -> Option<Span> {
		self.span().map(|x| x.start())
	}

	//----------------------------------------------------------------------------------------------------------------//
	// Ranges
	//----------------------------------------------------------------------------------------------------------------//

	pub fn range(&self) -> NodeRange {
		self.range.sub_range(self.index..)
	}

	pub fn range_to(&self, other: &NodeStream) -> NodeRange {
		assert!(std::ptr::eq(self.range.as_slice(), other.range.as_slice()));
		assert!(self.index <= other.index);
		self.range.sub_range(self.index..other.index)
	}

	pub fn range_from(&self, other: &NodeStream) -> NodeRange {
		other.range_to(self)
	}

	//----------------------------------------------------------------------------------------------------------------//
	// Parse helpers
	//----------------------------------------------------------------------------------------------------------------//

	pub fn to_list(&mut self) -> NodeList {
		let list = self.range.to_list();
		self.index = self.range.len();
		list
	}

	pub fn read_if<P: FnOnce(&Node) -> bool>(&mut self, predicate: P) -> Option<Node> {
		if self.peek().map(predicate) == Some(true) {
			self.read()
		} else {
			None
		}
	}

	pub fn read_map<T, P: FnOnce(Node) -> Option<T>>(&mut self, predicate: P) -> Option<T> {
		if let Some(result) = self.peek().cloned().and_then(predicate) {
			self.skip(1);
			Some(result)
		} else {
			None
		}
	}

	pub fn read_symbol(&mut self, symbol: &str) -> bool {
		self.read_if(|x| x.symbol() == Some(symbol)).is_some()
	}

	pub fn read_map_symbol<T, P: FnOnce(&str) -> Option<T>>(&mut self, predicate: P) -> Option<T> {
		self.read_map(|x| x.symbol().and_then(predicate))
	}

	pub fn skip_empty(&mut self) {
		loop {
			if !(self.skip_comments() || self.read_if(|x| x.is_break()).is_some()) {
				break;
			}
		}
	}

	pub fn skip_comments(&mut self) -> bool {
		let mut skipped = false;
		while let Some(..) = self.read_if(|x| x.is::<Comment>()) {
			skipped = true;
		}
		skipped
	}
}

//====================================================================================================================//
// Traits
//====================================================================================================================//

impl Iterator for NodeStream {
	type Item = Node;

	fn next(&mut self) -> Option<Self::Item> {
		self.read()
	}
}

impl Index<usize> for NodeStream {
	type Output = Node;

	fn index(&self, index: usize) -> &Self::Output {
		self.lookahead(index).expect("index out of bounds")
	}
}

//====================================================================================================================//
// Tests
//====================================================================================================================//

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn basic_lexing() {
		let input = open("1 + 2 * 3\n4");
		let nodes = input.collect::<Vec<_>>();
		assert!(nodes.len() == 7);
		assert_eq!(nodes[0].get_integer(), Some(Integer(1)));
		assert_eq!(nodes[1].get_token(), Some(&Token::Symbol("+")));
		assert_eq!(nodes[2].get_integer(), Some(Integer(2)));
		assert_eq!(nodes[3].get_token(), Some(&Token::Symbol("*")));
		assert_eq!(nodes[4].get_integer(), Some(Integer(3)));
		assert_eq!(nodes[5].get_token(), Some(&Token::Break));
		assert_eq!(nodes[6].get_integer(), Some(Integer(4)));
	}

	fn open(input: &'static str) -> NodeStream {
		let input = Input::from(input);
		let mut scanner = Scanner::default();
		scanner.add_matcher(IntegerMatcher);
		scanner.add_symbol("+", Token::Symbol("+"));
		scanner.add_symbol("-", Token::Symbol("-"));
		scanner.add_symbol("*", Token::Symbol("*"));
		scanner.add_symbol("/", Token::Symbol("/"));

		let mut errors = Errors::new();
		let list = NodeList::tokenize(input, &mut scanner, &mut errors);
		if !errors.empty() {
			println!("{}", errors);
			panic!("Token parsing generated errors");
		}
		list.into_iter()
	}
}
