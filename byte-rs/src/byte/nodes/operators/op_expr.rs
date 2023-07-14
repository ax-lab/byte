use std::collections::BTreeMap;

use super::*;

#[derive(Default, Clone, Debug, Eq, PartialEq, Hash)]
pub struct OperatorSet {
	set: OpMap<Operator>,
}

impl OperatorSet {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn add(&mut self, op: Operator) {
		assert!(!self.set.contains(op.symbol()));
		self.set.add(op.symbol().clone(), op)
	}

	pub fn register_symbols(&self, matcher: &mut Matcher) {
		self.set.register_symbols(matcher)
	}
}

impl ParseExpr for OperatorSet {
	type Op = Operator;

	fn get_operator(&self, node: &Node) -> Option<Self::Op> {
		self.set.op_for_node(node)
	}
}

impl IsNodeOperator for OperatorSet {
	fn can_apply(&self, node: &Node) -> bool {
		node.has_expr(self)
	}

	fn eval(&self, ctx: &mut EvalContext, node: &mut Node) -> Result<()> {
		node.parse_expr(ctx, self)
	}
}

//====================================================================================================================//
// Operator
//====================================================================================================================//

#[derive(Clone, Default, Eq, PartialEq)]
pub struct Operator {
	can_value: bool,
	symbol: Symbol,
	prefix: Option<(UnaryOp, OpPrecedence)>,
	posfix: Option<(UnaryOp, OpPrecedence)>,
	binary: Option<(BinaryOp, OpPrecedence, Grouping)>,
}

impl Operator {
	pub fn new(symbol: Symbol) -> Self {
		let mut output = Self::default();
		output.symbol = symbol;
		output
	}

	pub fn new_prefix(symbol: Symbol, op: UnaryOp, precedence: OpPrecedence) -> Self {
		Self::new(symbol).and_prefix(op, precedence)
	}

	pub fn new_posfix(symbol: Symbol, op: UnaryOp, precedence: OpPrecedence) -> Self {
		Self::new(symbol).and_posfix(op, precedence)
	}

	pub fn new_binary(symbol: Symbol, op: BinaryOp, precedence: OpPrecedence, grouping: Grouping) -> Self {
		Self::new(symbol).and_binary(op, precedence, grouping)
	}

	pub fn and_binary(mut self, op: BinaryOp, precedence: OpPrecedence, grouping: Grouping) -> Self {
		self.binary = Some((op, precedence, grouping));
		self
	}

	pub fn and_prefix(mut self, op: UnaryOp, precedence: OpPrecedence) -> Self {
		self.prefix = Some((op, precedence));
		self
	}

	pub fn and_posfix(mut self, op: UnaryOp, precedence: OpPrecedence) -> Self {
		self.posfix = Some((op, precedence));
		self
	}

	pub fn symbol(&self) -> &Symbol {
		&self.symbol
	}
}

impl IsOperator for Operator {
	type Precedence = OpPrecedence;

	fn prefix(&self) -> Option<OpPrecedence> {
		self.prefix.map(|x| x.1)
	}

	fn posfix(&self) -> Option<OpPrecedence> {
		self.posfix.map(|x| x.1)
	}

	fn binary(&self) -> Option<(OpPrecedence, Grouping)> {
		self.binary.map(|(_, p, g)| (p, g))
	}

	fn can_be_value(&self) -> bool {
		self.can_value
	}

	fn node_prefix(&self, ctx: &mut EvalContext, op: Node, arg: Node, span: Span) -> Result<Node> {
		let _ = op;
		let node = NodeValue::UnaryOp(self.prefix.unwrap().0, arg).at(ctx.scope_handle(), span);
		Ok(node)
	}

	fn node_posfix(&self, ctx: &mut EvalContext, op: Node, arg: Node, span: Span) -> Result<Node> {
		let _ = op;
		let node = NodeValue::UnaryOp(self.posfix.unwrap().0, arg).at(ctx.scope_handle(), span);
		Ok(node)
	}

	fn node_binary(&self, ctx: &mut EvalContext, op: Node, lhs: Node, rhs: Node, span: Span) -> Result<Node> {
		let _ = op;
		let node = NodeValue::BinaryOp(self.binary.unwrap().0, lhs, rhs).at(ctx.scope_handle(), span);
		Ok(node)
	}
}

impl Display for Operator {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		write!(f, "`{}`", self.symbol)
	}
}

impl Hash for Operator {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.symbol.hash(state)
	}
}

//====================================================================================================================//
// OpMap
//====================================================================================================================//

#[derive(Default, Clone)]
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

	pub fn register_symbols(&self, matcher: &mut Matcher) {
		for symbol in self.map.keys() {
			matcher.add_symbol(symbol.as_str())
		}
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
