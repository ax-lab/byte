use std::collections::VecDeque;

use crate::core::input::*;
use crate::lexer::*;
use crate::vm::operators::*;

use super::*;

#[derive(Debug)]
pub struct Raw {
	expr: NodeExprList,
}

has_traits!(Raw);

impl Raw {
	pub fn new(list: Vec<Node>, scope: Scope) -> Self {
		Self {
			expr: NodeExprList::new(list, scope),
		}
	}
}

impl IsNode for Raw {
	fn eval(&mut self, errors: &mut ErrorList) -> NodeEval {
		todo!()
	}
}

impl std::fmt::Display for Raw {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{self:?}")
	}
}

#[derive(Debug)]
pub enum RawExpr {
	Unary(OpUnary, Node),
	Binary(OpBinary, Node, Node),
	Ternary(OpTernary, Node, Node, Node),
}

has_traits!(RawExpr);

impl IsNode for RawExpr {
	fn eval(&mut self, errors: &mut ErrorList) -> NodeEval {
		todo!()
	}
}

impl std::fmt::Display for RawExpr {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{self:?}")
	}
}

//----------------------------------------------------------------------------//
// Expression parsing
//----------------------------------------------------------------------------//

pub struct NodeExprList {
	scope: Scope,
	list: Vec<Node>,
	next: usize,
	ops: VecDeque<(Op, Node)>,
	values: VecDeque<Node>,
	errors: ErrorList,
}

impl std::fmt::Debug for NodeExprList {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:?}", self.list)
	}
}

impl NodeExprList {
	pub fn new(list: Vec<Node>, scope: Scope) -> Self {
		Self {
			scope,
			list,
			next: 0,
			ops: Default::default(),
			values: Default::default(),
			errors: Default::default(),
		}
	}

	pub fn reduce(&mut self) -> NodeEval {
		if self.next >= self.list.len() {
			return self.check_pending();
		}

		loop {
			while let Some(op) = self.get_unary_pre() {
				// the unary operator doesn't affect other operators on the stack
				// because it binds forward to the next operator
				let op = Op::Unary(op);
				self.push_op(op, self.next().clone());
				self.advance();
			}

			let next = self.next().clone();
			match self.is_value() {
				Some(true) => {
					self.values.push_back(next);
					self.advance();
				}
				Some(false) => {
					self.errors.at(next.span(), "expected a value expression");
					return NodeEval::Complete;
				}
				None => {
					return NodeEval::DependsOn(vec![next]);
				}
			}

			// TODO: posfix operators (always pop themselves)

			// Ternary and binary operators work similarly, but the ternary will
			// parse the middle expression as parenthesized.
			if let Some((op, end)) = self.get_ternary() {
				let node = self.next().clone();
				self.advance();
				let op = Op::Ternary(op);
				self.push_op(op, node.clone());

				if let Some(index) = self.find_symbol(end) {
					let mut tail = self.list.split_off(index + 1);

					// Create a raw with the sub expression and append it as a node
					let expr = self.list.split_off(self.next);
					let expr = Raw::new(expr, self.scope.clone());
					let expr = Node::new(expr);
					self.list.push(expr.clone());
					self.list.append(&mut tail);
				} else {
					self.errors.at(
						node.span(),
						format!("symbol `{end}` for ternary operator {node} not found"),
					);
					return NodeEval::Complete;
				}
			} else if let Some(op) = self.get_binary() {
				let node = self.next().clone();
				let op = Op::Binary(op);
				self.push_op(op, node);
				self.advance();
			} else {
				break;
			}
		}

		// pop any remaining operators on the stack.
		while self.ops.len() > 0 {
			self.pop_stack();
		}

		// check that there was no unparsed portion of the expression
		if self.next < self.values.len() {
			if self.errors.empty() {
				self.errors
					.at(self.values[self.next].span(), "expected end of expression");
			}
			return NodeEval::Complete;
		}

		if self.values.len() == 0 {
			if self.errors.empty() {
				let sta = self.list.first().and_then(|x| x.span());
				let end = self.list.last().and_then(|x| x.span());
				let span = Span::from_range(sta, end);
				self.errors.at(span, "invalid expression");
			}
			NodeEval::Complete
		} else {
			assert!(self.values.len() == 1);
			let expr = self.values.pop_back().unwrap();
			NodeEval::FromNode(expr)
		}
	}

	/// If there are nodes pending evaluation return [`NodeEval::DependsOn`],
	/// otherwise returns [`NodeEval::Complete`].
	///
	/// Expression parsing is greedy. It parses a node as soon as it can be
	/// identified as an expression part. As such, nodes may remain pending
	/// even after the parsing is complete.
	fn check_pending(&self) -> NodeEval {
		let pending = self
			.list
			.iter()
			.filter(|x| x.is_done())
			.cloned()
			.collect::<Vec<_>>();
		if pending.len() > 0 {
			NodeEval::DependsOn(pending)
		} else {
			NodeEval::Complete
		}
	}

	//------------------------------------------------------------------------//
	// Stack manipulation
	//------------------------------------------------------------------------//

	/// Push a new operator onto the stack.
	fn push_op(&mut self, op: Op, node: Node) {
		while let Some((top, ..)) = self.ops.back() {
			if top < &op {
				self.pop_stack();
			} else {
				break;
			}
		}
		self.ops.push_back((op, node));
	}

	/// Pop a single operator and its operands from the stack and push the
	/// resulting operation.
	fn pop_stack(&mut self) {
		let ops = &mut self.ops;
		let values = &mut self.values;
		let (op, op_node) = ops.pop_back().unwrap();
		match op {
			Op::Unary(op) => {
				let expr = values.pop_back().unwrap();
				let span = Node::get_span(&op_node, &expr);
				let expr = RawExpr::Unary(op, expr);
				let expr = Node::new_at(expr, span);
				values.push_back(expr);
			}
			Op::Binary(op) => {
				let rhs = values.pop_back().unwrap();
				let lhs = values.pop_back().unwrap();
				let span = Node::get_span(&lhs, &rhs);
				let expr = RawExpr::Binary(op, lhs, rhs);
				let expr = Node::new_at(expr, span);
				values.push_back(expr);
			}
			Op::Ternary(op) => {
				let c = values.pop_back().unwrap();
				let b = values.pop_back().unwrap();
				let a = values.pop_back().unwrap();
				let span = Node::get_span(&a, &c);
				let expr = RawExpr::Ternary(op, a, b, c);
				let expr = Node::new_at(expr, span);
				values.push_back(expr);
			}
		}
	}

	//------------------------------------------------------------------------//
	// Helpers
	//------------------------------------------------------------------------//

	pub fn next(&self) -> &Node {
		&self.list[self.next]
	}

	pub fn advance(&mut self) {
		self.next += 1;
	}

	pub fn skip_symbol(&mut self, expected: &'static str) -> bool {
		if self.is_symbol_at(self.next, expected) {
			self.advance();
			true
		} else {
			false
		}
	}

	pub fn find_symbol(&self, symbol: &str) -> Option<usize> {
		for i in self.next..self.list.len() {
			if self.is_symbol_at(i, symbol) {
				return Some(i);
			}
		}
		None
	}

	pub fn is_symbol_at(&self, index: usize, symbol: &str) -> bool {
		let node = &self.list[index];
		if let Some(node) = node.get::<Atom>() {
			node.symbol() == Some(symbol)
		} else {
			false
		}
	}

	pub fn is_value(&mut self) -> Option<bool> {
		let node = self.next();
		let next = node.val();
		let next = next.read().unwrap();
		let next = &**next;
		let expr = to_trait!(next, IsExprValueNode);
		if let Some(expr) = expr {
			expr.is_value()
		} else if node.is_done() {
			Some(false)
		} else {
			None
		}
	}

	pub fn get_unary_pre(&self) -> Option<OpUnary> {
		let next = self.next().val();
		let next = next.read().unwrap();
		let next = &**next;
		let node = to_trait!(next, IsOperatorNode);
		node.and_then(|x| x.get_unary_pre())
	}

	pub fn get_binary(&self) -> Option<OpBinary> {
		let next = self.next().val();
		let next = next.read().unwrap();
		let next = &**next;
		let node = to_trait!(next, IsOperatorNode);
		node.and_then(|x| x.get_binary())
	}

	pub fn get_ternary(&mut self) -> Option<(OpTernary, &'static str)> {
		let next = self.next().val();
		let next = next.read().unwrap();
		let next = &**next;
		let node = to_trait!(next, IsOperatorNode);
		node.and_then(|x| x.get_ternary())
	}
}
