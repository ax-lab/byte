use super::*;

/// Trait implemented by operators that can be used with [`ParseExpr`].
pub trait IsOperator: Clone + Display {
	type Precedence: Ord + PartialOrd;

	/// If this operator can be applied as a unary prefix,
	/// return its precedence.
	fn prefix(&self) -> Option<Self::Precedence>;

	/// If this operator can be applied as a unary posfix,
	/// return its precedence.
	fn posfix(&self) -> Option<Self::Precedence>;

	/// If this operator can be applied as a binary infix,
	/// return its precedence and grouping.
	fn binary(&self) -> Option<(Self::Precedence, Grouping)>;

	/// True if the [`Node`] for this operator is also a valid value.
	fn can_be_value(&self) -> bool;

	fn node_prefix(&self, ctx: &mut EvalContext, op: Node, arg: Node, span: Span) -> Result<Node>;
	fn node_posfix(&self, ctx: &mut EvalContext, op: Node, arg: Node, span: Span) -> Result<Node>;
	fn node_binary(&self, ctx: &mut EvalContext, op: Node, lhs: Node, rhs: Node, span: Span) -> Result<Node>;
}

/// Grouping for binary operators with [`IsOperator`].
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Grouping {
	Left,
	Right,
}

pub trait ParseOps {
	type Op: IsOperator;

	fn is_operator(&self, node: &Node) -> bool {
		self.get_operator(node).is_some()
	}

	fn get_operator(&self, node: &Node) -> Option<Self::Op>;
}

impl Node {
	pub fn has_ops<T: ParseOps>(&self, op: &T) -> bool {
		self.contains(|x| op.is_operator(x))
	}

	pub fn parse_ops<T: ParseOps>(&mut self, ctx: &mut EvalContext, op: &T) -> Result<()> {
		let mut errors = Errors::new();
		let mut expr = OpStack::new(ctx);

		for node in self.iter() {
			if let Some(op) = op.get_operator(&node) {
				let has_value = expr.has_value();
				if has_value && op.posfix().is_some() {
					expr.push_op(op, OperatorMode::Posfix, node).handle(&mut errors);
				} else if !has_value && op.prefix().is_some() {
					expr.push_op(op, OperatorMode::Prefix, node).handle(&mut errors);
				} else if op.binary().is_some() {
					let mode = OperatorMode::Binary;
					expr.push_op(op, mode, node).handle(&mut errors);
				} else if op.can_be_value() {
					expr.append_to_value(node);
				} else {
					// invalid cases
					if op.prefix().is_some() {
						expr.push_op(op, OperatorMode::Prefix, node)
					} else if op.posfix().is_some() {
						expr.push_op(op, OperatorMode::Posfix, node)
					} else {
						let error = Errors::from(format!("operator {op} is not valid"), node.span());
						Err(error)
					}
					.handle(&mut errors);
				}
			} else {
				expr.append_to_value(node);
			}
			if !errors.empty() {
				break;
			}
		}

		if errors.len() > 0 {
			Err(errors)
		} else {
			let nodes = expr.finish()?;
			self.replace_all(nodes);
			Ok(())
		}
	}
}

//====================================================================================================================//
// Internals
//====================================================================================================================//

/// Application mode for operators with [`IsOperator`].
#[derive(Copy, Clone, Eq, PartialEq)]
enum OperatorMode {
	Prefix,
	Posfix,
	Binary,
}

impl OperatorMode {
	pub fn is_unary(&self) -> bool {
		match self {
			OperatorMode::Prefix => true,
			OperatorMode::Posfix => true,
			OperatorMode::Binary => false,
		}
	}

	pub fn is_binary(&self) -> bool {
		!self.is_unary()
	}
}

/// Helper to parse an expression.
struct OpStack<'a, T: IsOperator> {
	ctx: &'a mut EvalContext,
	ops: VecDeque<(T, OperatorMode, Node)>,
	values: VecDeque<Vec<Node>>,
}

impl<'a, T: IsOperator> OpStack<'a, T> {
	pub fn new(ctx: &'a mut EvalContext) -> Self {
		let mut output = Self {
			ctx,
			ops: Default::default(),
			values: Default::default(),
		};
		output.values.push_back(Vec::new());
		output
	}

	pub fn has_value(&self) -> bool {
		self.values.back().unwrap().len() > 0
	}

	pub fn append_to_value(&mut self, node: Node) {
		self.values.back_mut().unwrap().push(node);
	}

	pub fn push_op(&mut self, op: T, mode: OperatorMode, node: Node) -> Result<()> {
		let (prec, ..) = Self::precedence(&op, &mode);

		// sanity check the operator application
		let error = match mode {
			OperatorMode::Prefix => {
				if self.has_value() {
					Some(format!("unexpected prefix operator {op}"))
				} else {
					None
				}
			}
			OperatorMode::Posfix => {
				if !self.has_value() {
					Some(format!("posfix operator {op} missing operand"))
				} else {
					None
				}
			}
			OperatorMode::Binary => {
				if !self.has_value() {
					Some(format!("binary operator {op} missing left operand"))
				} else {
					None
				}
			}
		};
		if let Some(error) = error {
			return Err(Errors::from(error, node.span()));
		}

		// prefix operators never pop the stack because they bind ahead
		if mode != OperatorMode::Prefix {
			// pop any operation with a higher precedence
			while let Some((op_b, mode_b, ..)) = self.ops.back() {
				// lower precedence value have higher priority
				let (prec_b, grp_b) = Self::precedence(&op_b, mode_b);
				let pop = prec_b < prec || (prec_b == prec && grp_b == Grouping::Left);
				if pop {
					self.pop_operator()?;
				} else {
					break;
				}
			}
		}

		self.ops.push_back((op, mode, node));

		if mode.is_binary() {
			// an infix operator will start a new value
			self.values.push_back(Default::default());
		} else if mode == OperatorMode::Posfix {
			// posfix operators apply immediately
			self.pop_operator()?;
		}

		Ok(())
	}

	pub fn finish(&mut self) -> Result<Vec<Node>> {
		while self.ops.len() > 0 {
			self.pop_operator()?;
		}
		assert!(self.values.len() == 1);

		let nodes = self.values.pop_back().unwrap();
		Ok(nodes)
	}

	fn push_new_value(&mut self, node: Node) {
		self.values.push_back(vec![node]);
	}

	fn pop_operator(&mut self) -> Result<()> {
		let (op, mode, node) = self.ops.pop_back().unwrap();
		if mode.is_unary() {
			let arg = self.pop_value();
			let span = Span::merge(arg.span(), node.span());
			let node = if mode == OperatorMode::Posfix {
				op.node_posfix(self.ctx, node, arg, span)?
			} else {
				op.node_prefix(self.ctx, node, arg, span)?
			};
			self.push_new_value(node);
		} else {
			let rhs = self.pop_value();
			let lhs = self.pop_value();
			if rhs.len() == 0 {
				let error = Errors::from(format!("right-hand operand expected for {op}"), node.span());
				return Err(error);
			}
			let span = Span::merge(lhs.span(), rhs.span());
			let node = op.node_binary(self.ctx, node, lhs, rhs, span)?;
			self.push_new_value(node);
		}
		Ok(())
	}

	fn pop_value(&mut self) -> Node {
		let list = self.values.pop_back().unwrap();
		let node = Node::raw(list, self.ctx.scope_handle());
		node
	}

	fn precedence(op: &T, mode: &OperatorMode) -> (T::Precedence, Grouping) {
		match mode {
			OperatorMode::Prefix => (op.prefix().unwrap(), Grouping::Right),
			OperatorMode::Posfix => (op.posfix().unwrap(), Grouping::Left),
			OperatorMode::Binary => op.binary().unwrap(),
		}
	}
}
