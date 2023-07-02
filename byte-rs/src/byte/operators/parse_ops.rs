use std::collections::BTreeMap;

use super::*;

//====================================================================================================================//
// OpMap
//====================================================================================================================//

#[derive(Clone)]
pub struct OpMap<T: Clone> {
	map: BTreeMap<Name, T>,
}

impl<T: Clone> OpMap<T> {
	pub fn new() -> Self {
		Self {
			map: Default::default(),
		}
	}

	pub fn add(&mut self, symbol: Name, op: T) {
		self.map.insert(symbol, op);
	}

	pub fn contains(&self, symbol: &Name) -> bool {
		self.map.contains_key(&symbol)
	}

	pub fn op_for_node(&self, node: &NodeData) -> Option<T> {
		node.name().and_then(|symbol| self.map.get(&symbol)).cloned()
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
pub struct ParseBinaryOp(pub OpMap<BinaryOp>, pub Precedence, pub Grouping);

impl IsOperator for ParseBinaryOp {
	fn precedence(&self) -> Precedence {
		self.1
	}

	fn predicate(&self, node: &Node) -> bool {
		match node {
			Node::Word(name) | Node::Symbol(name) => self.0.contains(name),
			_ => false,
		}
	}

	fn apply(&self, context: &mut OperatorContext, errors: &mut Errors) {
		let _ = errors;
		let mut new_lists = Vec::new();

		let is_op = |node: &NodeData| {
			if let Some(name) = node.name() {
				self.0.contains(&name)
			} else {
				false
			}
		};

		let fold = |lhs: NodeList, op: NodeData, rhs: NodeList| {
			new_lists.push(lhs.clone());
			new_lists.push(rhs.clone());
			let op = self.0.op_for_node(&op).unwrap();
			let span = lhs.span();
			let node = Node::BinaryOp(op, lhs, rhs);
			node.at(span)
		};

		if self.2 == Grouping::Left {
			context.nodes().fold_last(is_op, fold);
		} else {
			context.nodes().fold_first(is_op, fold);
		}

		for it in new_lists {
			context.resolve_nodes(&it);
		}
	}
}

//====================================================================================================================//
// ParseUnaryOp
//====================================================================================================================//

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ParseUnaryPrefixOp(pub OpMap<UnaryOp>, pub Precedence);

impl IsOperator for ParseUnaryPrefixOp {
	fn precedence(&self) -> Precedence {
		self.1
	}

	fn can_apply(&self, nodes: &NodeList) -> bool {
		match nodes.get(0).as_ref().map(|x| x.get()) {
			Some(Node::Word(name) | Node::Symbol(name)) => self.0.contains(name),
			_ => false,
		}
	}

	fn apply(&self, context: &mut OperatorContext, errors: &mut Errors) {
		let _ = errors;
		let nodes = context.nodes();
		let op = self.0.op_for_node(&nodes.get(0).unwrap()).unwrap();
		let arg = nodes.slice(1..);
		let new = Node::UnaryOp(op, arg.clone()).at(nodes.span());
		nodes.replace_all(vec![new]);
		context.resolve_nodes(&arg);
	}
}
