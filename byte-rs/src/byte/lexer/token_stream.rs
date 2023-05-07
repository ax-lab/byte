use super::*;

pub trait TokenStream {
	fn lookahead(&self, n: usize) -> Option<Node>;

	fn read(&mut self, errors: &mut Errors) -> Option<Node>;

	fn next(&self) -> Option<Node> {
		self.lookahead(0)
	}
}

#[derive(Clone)]
pub struct ListTokenStream {
	list: Arc<VecDeque<Node>>,
	next: usize,
}

impl TokenStream for ListTokenStream {
	fn lookahead(&self, n: usize) -> Option<Node> {
		self.list.get(self.next + n).cloned()
	}

	fn read(&mut self, _errors: &mut Errors) -> Option<Node> {
		if let Some(next) = self.list.get(self.next) {
			self.next += 1;
			Some(next.clone())
		} else {
			None
		}
	}
}

#[derive(Clone)]
pub struct InputTokenStream {
	cursor: Cursor,
	scanner: Scanner,
	next: Arc<RwLock<Arc<VecDeque<(Node, Cursor, Errors)>>>>,
}

impl InputTokenStream {
	pub fn new(input: Cursor, scanner: Scanner) -> Self {
		Self {
			cursor: input,
			scanner,
			next: Default::default(),
		}
	}

	pub fn config<F: FnOnce(&mut Scanner)>(&mut self, config: F) {
		config(&mut self.scanner);
		self.flush_next();
	}

	pub fn lookahead(&self, n: usize) -> Option<Node> {
		{
			let next = self.next.read().unwrap();
			if let Some((node, ..)) = next.get(n) {
				return Some(node.clone());
			} else if let Some((_, cursor, ..)) = next.back() {
				if cursor.is_end() {
					return None;
				}
			}
		}

		let mut next = self.next.write().unwrap();
		let next = Arc::make_mut(&mut next);
		let (mut cursor, mut errors) = next
			.back()
			.map(|(_, cursor, errors)| (cursor.clone(), errors.clone()))
			.unwrap_or_else(|| (self.cursor.clone(), Errors::default()));
		while n >= next.len() {
			let node = self.scanner.read(&mut cursor, &mut errors);
			if let Some(node) = node {
				next.push_back((node, cursor.clone(), errors.clone()));
			} else {
				break;
			}
		}
		next.back().map(|x| x.0.clone())
	}

	pub fn read(&mut self, errors: &mut Errors) -> Option<Node> {
		let mut next = self.next.write().unwrap();
		let next = Arc::make_mut(&mut next);
		if let Some((node, cursor, node_errors)) = next.pop_front() {
			if node_errors.len() > 0 {
				errors.append(node_errors);
			}
			self.cursor = cursor;
			return Some(node);
		} else {
			self.scanner.read(&mut self.cursor, errors)
		}
	}

	pub fn skip(&mut self, count: usize, errors: &mut Errors) {
		for _ in 0..count {
			self.read(errors);
		}
	}

	fn flush_next(&mut self) {
		let mut next = self.next.write().unwrap();
		let next = Arc::make_mut(&mut next);
		next.clear();
	}
}

impl TokenStream for InputTokenStream {
	fn lookahead(&self, n: usize) -> Option<Node> {
		Self::lookahead(self, n)
	}

	fn read(&mut self, errors: &mut Errors) -> Option<Node> {
		Self::read(self, errors)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::lang::*;

	#[test]
	fn basic_lexing() {
		let mut input = open("1 + 2 * 3\n4");
		let mut nodes = Vec::new();
		let mut errors = Errors::new();
		while let Some(node) = input.read(&mut errors) {
			nodes.push(node);
		}

		assert!(errors.empty());

		assert!(nodes.len() == 7);
		assert_eq!(nodes[0].get_integer(), Some(Integer(1)));
		assert_eq!(nodes[1].get_token(), Some(&Token::Symbol("+")));
		assert_eq!(nodes[2].get_integer(), Some(Integer(2)));
		assert_eq!(nodes[3].get_token(), Some(&Token::Symbol("*")));
		assert_eq!(nodes[4].get_integer(), Some(Integer(3)));
		assert_eq!(nodes[5].get_token(), Some(&Token::Break));
		assert_eq!(nodes[6].get_integer(), Some(Integer(4)));
	}

	fn open(input: &str) -> InputTokenStream {
		let input = Input::from(input);
		let mut scanner = Scanner::new();
		scanner.add_matcher(IntegerMatcher);
		scanner.add_symbol("+", Token::Symbol("+"));
		scanner.add_symbol("-", Token::Symbol("-"));
		scanner.add_symbol("*", Token::Symbol("*"));
		scanner.add_symbol("/", Token::Symbol("/"));
		InputTokenStream::new(input.start(), scanner)
	}
}
