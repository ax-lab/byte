use std::{ops::*, sync::Arc};

use super::*;

#[derive(Clone)]
pub struct NodeStream {
	stream: Arc<Vec<Node>>,
	next: usize,
	end: usize,
}

impl NodeStream {
	pub(crate) fn new(list: Arc<Vec<Node>>, start: usize, end: usize) -> Self {
		Self {
			stream: list,
			next: start,
			end,
		}
	}

	pub fn len(&self) -> usize {
		self.end - self.next
	}

	pub fn peek(&self) -> Option<&Node> {
		self.lookahead(0)
	}

	pub fn lookahead(&self, n: usize) -> Option<&Node> {
		self.list().get(n)
	}

	pub fn read(&mut self) -> Option<Node> {
		self.peek().cloned().map(|x| {
			self.next += 1;
			x
		})
	}

	pub fn skip(&mut self, count: usize) {
		let next = self.next + count;
		self.next = std::cmp::min(next, self.end)
	}

	fn list(&self) -> &[Node] {
		&self.stream[self.next..self.end]
	}

	//----------------------------------------------------------------------------------------------------------------//
	// Parse helpers
	//----------------------------------------------------------------------------------------------------------------//

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
		&self.list()[index]
	}
}

impl Index<Range<usize>> for NodeStream {
	type Output = [Node];

	fn index(&self, index: Range<usize>) -> &Self::Output {
		&self.list()[index]
	}
}

impl Index<RangeInclusive<usize>> for NodeStream {
	type Output = [Node];

	fn index(&self, index: RangeInclusive<usize>) -> &Self::Output {
		&self.list()[index]
	}
}

impl Index<RangeFrom<usize>> for NodeStream {
	type Output = [Node];

	fn index(&self, index: RangeFrom<usize>) -> &Self::Output {
		&self.list()[index]
	}
}

impl Index<RangeTo<usize>> for NodeStream {
	type Output = [Node];

	fn index(&self, index: RangeTo<usize>) -> &Self::Output {
		&self.list()[index]
	}
}
impl Index<RangeToInclusive<usize>> for NodeStream {
	type Output = [Node];

	fn index(&self, index: RangeToInclusive<usize>) -> &Self::Output {
		&self.list()[index]
	}
}
impl Index<RangeFull> for NodeStream {
	type Output = [Node];

	fn index(&self, index: RangeFull) -> &Self::Output {
		&self.list()[index]
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
		let mut scanner = Scanner::new();
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
