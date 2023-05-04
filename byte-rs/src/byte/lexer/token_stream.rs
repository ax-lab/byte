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
