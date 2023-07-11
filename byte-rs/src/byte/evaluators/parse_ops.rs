use std::collections::BTreeMap;

use super::*;

//====================================================================================================================//
// OpMap
//====================================================================================================================//

#[derive(Clone)]
pub struct OpMap<T: Clone> {
	map: BTreeMap<Symbol, T>,
}

impl<T: Clone> OpMap<T> {
	pub fn new() -> Self {
		Self {
			map: Default::default(),
		}
	}

	pub fn add(&mut self, symbol: Symbol, op: T) {
		self.map.insert(symbol, op);
	}

	pub fn contains(&self, symbol: &Symbol) -> bool {
		self.map.contains_key(&symbol)
	}

	pub fn op_for_node(&self, node: &Node) -> Option<T> {
		node.symbol().and_then(|symbol| self.map.get(&symbol)).cloned()
	}
}

impl<T: PartialEq + Clone> PartialEq for OpMap<T> {
	fn eq(&self, other: &Self) -> bool {
		if self.map.len() == other.map.len() {
			for (key, val) in self.map.iter() {
				if other.map.get(key) != Some(val) {
					return false;
				}
			}
			true
		} else {
			false
		}
	}
}

impl<T: Eq + Clone> Eq for OpMap<T> {}

impl<T: Hash + Clone> Hash for OpMap<T> {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		for (key, val) in self.map.iter() {
			key.hash(state);
			val.hash(state);
		}
	}
}

impl<T: Clone> Debug for OpMap<T> {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		write!(f, "Ops(")?;
		for (n, key) in self.map.keys().enumerate() {
			if n > 0 {
				write!(f, ", ")?;
			}
			write!(f, "{key}")?;
		}
		write!(f, ")")
	}
}

//====================================================================================================================//
// ParseBinaryOp
//====================================================================================================================//

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ParseBinaryOp(pub OpMap<BinaryOp>, pub Grouping);

impl IsNodeOperator for ParseBinaryOp {
	fn can_apply(&self, nodes: &NodeList) -> bool {
		nodes.contains(|node| match node.bit() {
			Bit::Token(Token::Word(symbol) | Token::Symbol(symbol)) => self.0.contains(symbol),
			_ => false,
		})
	}

	fn apply(&self, nodes: &mut NodeList, ctx: &mut EvalContext) -> Result<()> {
		let mut new_lists = Vec::new();

		let is_op = |node: &Node| {
			if let Some(symbol) = node.symbol() {
				self.0.contains(&symbol)
			} else {
				false
			}
		};

		let fold = |lhs: NodeList, op: Node, rhs: NodeList| {
			new_lists.push(lhs.clone());
			new_lists.push(rhs.clone());
			let op = self.0.op_for_node(&op).unwrap();
			let span = lhs.span();
			Bit::BinaryOp(op, lhs, rhs).at(span)
		};

		if self.1 == Grouping::Left {
			nodes.fold_last(is_op, fold)
		} else {
			nodes.fold_first(is_op, fold)
		}

		for it in new_lists {
			ctx.resolve_nodes(&it);
		}

		Ok(())
	}
}

//====================================================================================================================//
// ParseUnaryOp
//====================================================================================================================//

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ParseUnaryPrefixOp(pub OpMap<UnaryOp>);

impl IsNodeOperator for ParseUnaryPrefixOp {
	fn can_apply(&self, nodes: &NodeList) -> bool {
		match nodes.get(0).as_ref().map(|x| x.bit()) {
			Some(Bit::Token(Token::Word(symbol) | Token::Symbol(symbol))) => self.0.contains(symbol),
			_ => false,
		}
	}

	fn apply(&self, nodes: &mut NodeList, ctx: &mut EvalContext) -> Result<()> {
		let op = self.0.op_for_node(&nodes.get(0).unwrap()).unwrap();
		let arg = nodes.slice(1..);
		let new = Bit::UnaryOp(op, arg.clone()).at(nodes.span());
		nodes.replace_all(vec![new]);
		ctx.resolve_nodes(&arg);
		Ok(())
	}
}
