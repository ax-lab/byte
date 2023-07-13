use super::*;

impl NodeList {
	pub fn get_symbol(&self, index: usize) -> Option<Symbol> {
		let nodes = self.data.nodes.read().unwrap();
		nodes.get(index).and_then(|x| x.symbol())
	}

	pub fn test_at<P: FnOnce(&Node) -> bool>(&self, index: usize, predicate: P) -> bool {
		let nodes = self.data.nodes.read().unwrap();
		nodes.get(index).map(|x| predicate(x)).unwrap_or(false)
	}

	pub fn is_identifier(&self, index: usize) -> bool {
		self.test_at(index, |x| matches!(x.bit(), Bit::Token(Token::Word(..))))
	}

	pub fn is_keyword(&self, index: usize, word: &Symbol) -> bool {
		self.test_at(index, |x| x.is_keyword(word))
	}

	pub fn is_symbol(&self, index: usize, symbol: &Symbol) -> bool {
		self.test_at(index, |x| x.is_symbol(symbol))
	}

	/// Replace the entire list by the vector contents.
	pub fn replace_all(&mut self, vec: Vec<Node>) {
		self.write(|nodes| {
			*nodes = vec;
			true
		});
	}
}
