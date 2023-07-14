use super::*;

impl Node {
	pub fn contains<P: Fn(&Node) -> bool>(&self, predicate: P) -> bool {
		self.iter().any(|node| predicate(&node))
	}

	pub fn to_vec(&self) -> Vec<Node> {
		self.iter().collect()
	}

	pub fn get_symbol_at(&self, index: usize) -> Option<Symbol> {
		self.get(index).and_then(|x| x.symbol())
	}

	pub fn test_at<P: FnOnce(&Node) -> bool>(&self, index: usize, predicate: P) -> bool {
		let node = self.val();
		node.get(index).map(|x| predicate(x)).unwrap_or(false)
	}

	pub fn is_identifier(&self, index: usize) -> bool {
		self.test_at(index, |x| matches!(x.val(), NodeValue::Token(Token::Word(..))))
	}

	pub fn is_keyword_at(&self, index: usize, word: &Symbol) -> bool {
		self.test_at(index, |x| x.is_keyword(word))
	}

	pub fn is_symbol_at(&self, index: usize, symbol: &Symbol) -> bool {
		self.test_at(index, |x| x.is_symbol(symbol))
	}

	/// Replace the entire list by the vector contents.
	pub fn replace_all(&mut self, nodes: Vec<Node>) {
		let span = Span::from_node_vec(&nodes);
		let value = NodeValue::Raw(nodes.into());
		self.set_value(value, span);
	}

	pub fn rewrite<P: FnOnce(&mut Vec<Node>) -> bool>(&mut self, writer: P) {
		let mut nodes = self.to_vec();
		if writer(&mut nodes) {
			self.replace_all(nodes);
		}
	}

	pub fn rewrite_res<P: FnOnce(&mut Vec<Node>) -> Result<bool>>(&mut self, writer: P) -> Result<()> {
		let mut nodes = self.to_vec();
		if writer(&mut nodes)? {
			self.replace_all(nodes);
		}
		Ok(())
	}
}
