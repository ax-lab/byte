use super::*;

// TODO: get rid of unused or hyper-specific methods

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
		self.test_at(index, |x| x.is_word(word))
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

	pub fn fold_first<P: FnMut(&Node) -> bool, S: FnMut(NodeList, Node, NodeList) -> Node>(
		&mut self,
		mut fold: P,
		mut make_node: S,
	) {
		let scope = self.scope();
		self.write(|nodes| {
			let mut changed = false;
			{
				for i in 0..nodes.len() {
					if fold(&nodes[i]) {
						let lhs = nodes[0..i].to_vec();
						let cur = nodes[i].clone();
						let rhs = nodes[i + 1..].to_vec();
						let lhs = NodeList::new(scope.handle(), lhs);
						let rhs = NodeList::new(scope.handle(), rhs);
						let node = make_node(lhs, cur, rhs);
						*nodes = vec![node];
						changed = true;
						break;
					}
				}
			}

			changed
		});
	}

	pub fn fold_last<P: FnMut(&Node) -> bool, S: FnMut(NodeList, Node, NodeList) -> Node>(
		&mut self,
		mut fold: P,
		mut make_node: S,
	) {
		let scope = self.scope();
		self.write(|nodes| {
			let mut changed = false;
			{
				for i in (0..nodes.len()).rev() {
					if fold(&nodes[i]) {
						let lhs = nodes[0..i].to_vec();
						let cur = nodes[i].clone();
						let rhs = nodes[i + 1..].to_vec();
						let lhs = NodeList::new(scope.handle(), lhs);
						let rhs = NodeList::new(scope.handle(), rhs);
						let node = make_node(lhs, cur, rhs);
						*nodes = vec![node];
						changed = true;
						break;
					}
				}
			}

			changed
		});
	}
}
