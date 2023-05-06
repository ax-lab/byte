use super::*;

#[derive(Clone)]
pub struct TokenStream {
	cursor: Cursor,
	scanner: Scanner,
	next: Arc<RwLock<Arc<VecDeque<(Node, Cursor, Errors)>>>>,
}

impl TokenStream {
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

	pub fn lookahead(&self, n: usize) -> Node {
		{
			let next = self.next.read().unwrap();
			if let Some((node, ..)) = next.get(n) {
				return node.clone();
			} else if let Some((last, ..)) = next.back() {
				if last.is(Token::EndOfInput) {
					return last.clone();
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
			let is_none = node.is(Token::EndOfInput);
			next.push_back((node, cursor.clone(), errors.clone()));
			if is_none {
				break;
			}
		}
		next.back().map(|x| x.0.clone()).unwrap()
	}

	pub fn next(&self) -> Node {
		self.lookahead(0)
	}

	pub fn read(&mut self, errors: &mut Errors) -> Node {
		let mut next = self.next.write().unwrap();
		let next = Arc::make_mut(&mut next);
		if let Some((node, cursor, node_errors)) = next.pop_front() {
			if node_errors.len() > 0 {
				errors.append(node_errors);
			}
			self.cursor = cursor;
			return node;
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

#[cfg(test)]
mod tests {
	use super::*;
	use crate::lang::*;

	#[test]
	fn basic_lexing() {
		let mut input = open("1 + 2 * 3\n4");
		let mut nodes = Vec::new();
		let mut errors = Errors::new();
		loop {
			let node = input.read(&mut errors);
			if !node.is_end() {
				nodes.push(node);
			} else {
				break;
			}
		}

		assert!(nodes.len() == 7);
		assert_eq!(nodes[0].get_integer(), Some(Integer(1)));
		assert_eq!(nodes[1].get_token(), Some(Token::Symbol("+")));
		assert_eq!(nodes[2].get_integer(), Some(Integer(2)));
		assert_eq!(nodes[3].get_token(), Some(Token::Symbol("*")));
		assert_eq!(nodes[4].get_integer(), Some(Integer(3)));
		assert_eq!(nodes[5].get_token(), Some(Token::Break));
		assert_eq!(nodes[6].get_integer(), Some(Integer(4)));
	}

	fn open(input: &str) -> TokenStream {
		let input = Input::from(input);
		let mut scanner = Scanner::new();
		scanner.add_matcher(IntegerMatcher);
		scanner.add_symbol("+", Token::Symbol("+"));
		scanner.add_symbol("-", Token::Symbol("-"));
		scanner.add_symbol("*", Token::Symbol("*"));
		scanner.add_symbol("/", Token::Symbol("/"));
		TokenStream::new(input.start(), scanner)
	}
}
