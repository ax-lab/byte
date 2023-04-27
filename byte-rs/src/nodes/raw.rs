use std::collections::VecDeque;
use std::io::Write;

use crate::core::repr::*;
use crate::vm::operators::*;

use super::*;

/// Raw list of unprocessed atom nodes.
#[derive(Clone)]
pub struct Raw {
	expr: NodeExprList,
}

impl Raw {
	pub fn new(list: Vec<Node>) -> Self {
		Self {
			expr: NodeExprList::new(list),
		}
	}
}

has_traits!(Raw: IsNode, HasRepr);

impl IsNode for Raw {
	fn eval(&mut self) -> NodeEval {
		self.expr.reduce()
	}

	fn span(&self) -> Option<Span> {
		Node::span_from_list(&self.expr.list)
	}
}

impl HasRepr for Raw {
	fn output_repr(&self, output: &mut Repr) -> std::io::Result<()> {
		if self.expr.list.len() == 0 {
			return write!(output, "Raw()");
		}
		write!(output, "Raw(\n")?;
		{
			let mut output = output.indented();
			for it in self.expr.list.iter() {
				it.output_repr(&mut output)?;
				write!(output, "\n")?;
			}
		}
		write!(output, ")")?;
		Ok(())
	}
}

/// Raw expression node.
#[derive(Clone, PartialEq)]
pub enum RawExpr {
	Unary(OpUnary, Node),
	Binary(OpBinary, Node, Node),
	Ternary(OpTernary, Node, Node, Node),
}

has_traits!(RawExpr: IsNode, HasRepr);

impl IsNode for RawExpr {
	fn eval(&mut self) -> NodeEval {
		let mut deps = Vec::new();
		match self {
			RawExpr::Unary(_, a) => {
				if !a.is_done() {
					deps.push(a.clone());
				}
			}
			RawExpr::Binary(_, a, b) => {
				if !a.is_done() {
					deps.push(a.clone());
				}
				if !b.is_done() {
					deps.push(b.clone());
				}
			}
			RawExpr::Ternary(_, a, b, c) => {
				if !a.is_done() {
					deps.push(a.clone());
				}
				if !b.is_done() {
					deps.push(b.clone());
				}
				if !c.is_done() {
					deps.push(c.clone());
				}
			}
		}
		if deps.len() > 0 {
			NodeEval::DependsOn(deps)
		} else {
			NodeEval::Complete
		}
	}

	fn span(&self) -> Option<Span> {
		match self {
			RawExpr::Unary(_, a) => a.span(),
			RawExpr::Binary(_, a, b) => Node::span_from(a, b),
			RawExpr::Ternary(_, a, _, b) => Node::span_from(a, b),
		}
	}
}

impl HasRepr for RawExpr {
	fn output_repr(&self, output: &mut repr::Repr) -> std::io::Result<()> {
		let compact = output.is_compact();
		if output.is_debug() {
			match self {
				RawExpr::Unary(op, a) => {
					write!(output, "Unary::{op:?}(")?;
					if compact {
						write!(output, " ")?;
						a.output_repr(output)?;
					} else {
						let mut output = output.indented();
						write!(output, "\n")?;
						a.output_repr(&mut output)?;
						write!(output, "\n")?;
					}
					write!(output, ")")?;
				}
				RawExpr::Binary(op, a, b) => {
					write!(output, "Binary::{op:?}(")?;
					if compact {
						write!(output, " ")?;
						a.output_repr(output)?;
						write!(output, ", ")?;
						b.output_repr(output)?;
					} else {
						let mut output = output.indented();
						write!(output, "\n")?;
						a.output_repr(&mut output)?;
						write!(output, "\n")?;
						b.output_repr(&mut output)?;
						write!(output, "\n")?;
					}
					write!(output, ")")?;
				}
				RawExpr::Ternary(op, a, b, c) => {
					write!(output, "Ternary::{op:?}(")?;
					if compact {
						write!(output, " ")?;
						a.output_repr(output)?;
						write!(output, ", ")?;
						b.output_repr(output)?;
						write!(output, ", ")?;
						c.output_repr(output)?;
					} else {
						let mut output = output.indented();
						write!(output, "\n")?;
						a.output_repr(&mut output)?;
						write!(output, "\n")?;
						b.output_repr(&mut output)?;
						write!(output, "\n")?;
						c.output_repr(&mut output)?;
						write!(output, "\n")?;
					}
					write!(output, ")")?;
				}
			};
		} else {
			write!(output, "(")?;
			match self {
				RawExpr::Unary(op, a) => {
					if op.is_posfix() {
						a.output_repr(output)?;
						write!(output, "{op}")?;
					} else {
						write!(output, "{op}")?;
						a.output_repr(output)?;
					}
				}
				RawExpr::Binary(op, a, b) => {
					a.output_repr(output)?;
					write!(output, " {op} ")?;
					b.output_repr(output)?;
				}
				RawExpr::Ternary(op, a, b, c) => {
					let (s1, s2) = op.get_symbol();
					a.output_repr(output)?;
					write!(output, " {s1} ")?;
					b.output_repr(output)?;
					write!(output, " {s2} ")?;
					c.output_repr(output)?;
				}
			};
			write!(output, ")")?;
		}
		Ok(())
	}
}

//----------------------------------------------------------------------------//
// Expression parsing
//----------------------------------------------------------------------------//

#[derive(Clone)]
pub struct NodeExprList {
	list: Vec<Node>,
	next: usize,
	ops: VecDeque<(Op, Node)>,
	values: VecDeque<Node>,
	queued: VecDeque<Node>,
}

impl NodeExprList {
	pub fn new(list: Vec<Node>) -> Self {
		Self {
			list,
			next: 0,
			ops: Default::default(),
			values: Default::default(),
			queued: Default::default(),
		}
	}

	pub fn reduce(&mut self) -> NodeEval {
		if self.next >= self.list.len() && self.queued.len() == 0 {
			return self.check_pending();
		}

		loop {
			// Protection against an infinite macro expression expansion
			if self.queued.len() > 1024 && self.queued.len() > self.list.len() * 2 {
				self.list[0].clone().errors_mut().at(
					Node::span_from_list(&self.list),
					"expression expansion overflow",
				);
				return NodeEval::Complete;
			}

			// Only evaluate the expression when the next node is available.
			if let Some(next) = self.next() {
				if !next.is_done() {
					return NodeEval::DependsOn(vec![next.clone()]);
				}
			}

			// Check for a macro node
			if let Some(next) = self.next() {
				let next = next.clone();
				let next = next.val();
				if let Some(next) = get_trait!(&*next, IsMacroNode) {
					let nodes = Vec::from_iter(
						self.queued
							.iter()
							.chain(self.list[self.next..].iter())
							.cloned(),
					);
					if let Some((new_nodes, mut consumed)) = next.try_parse(&nodes) {
						assert!(consumed > 0);
						while consumed > 0 && self.queued.len() > 0 {
							self.queued.pop_front();
							consumed -= 1;
						}
						self.next += consumed;
						for it in new_nodes.into_iter().rev() {
							self.queued.push_front(it);
						}
						continue;
					}
				}
			}

			if let Some(op) = self.get_unary_pre() {
				// the unary operator doesn't affect other operators on the stack
				// because it binds forward to the next operator
				let op = Op::Unary(op);
				self.push_op(op, self.next().unwrap().clone());
				self.advance();
				continue;
			}

			let next = self.next().cloned();
			let saved = next.clone();
			if self.is_value() {
				self.values.push_back(next.unwrap());
				self.advance();
			} else {
				let mut errors = next.clone().or(self.list.last().cloned()).unwrap();
				let mut errors = errors.errors_mut();
				errors.at(
					next.and_then(|x| x.span()),
					format!(
						"expected a value expression -- {}",
						if let Some(next) = &saved {
							next.repr_for_msg()
						} else {
							format!("got none")
						}
					),
				);
				return NodeEval::Complete;
			}

			// TODO: posfix operators (always pop themselves)

			// Ternary and binary operators work similarly, but the ternary will
			// parse the middle expression as parenthesized.
			if let Some((op, end)) = self.get_ternary() {
				let node = self.next().unwrap().clone();
				self.advance();
				let op = Op::Ternary(op);
				self.push_op(op, node.clone());

				if let Some(index) = self.find_symbol(end) {
					let mut tail = self.list.split_off(index + 1);

					// Create a raw with the sub expression and append it as a node
					let expr = self.list.split_off(self.next);
					let expr = Raw::new(expr);
					let expr = Node::new(expr);
					self.list.push(expr.clone());
					self.list.append(&mut tail);
				} else {
					let mut errors = self.list[0].errors_mut();
					errors.at(
						node.span(),
						format!("symbol `{end}` for ternary operator {node} not found"),
					);
					return NodeEval::Complete;
				}
			} else if let Some(op) = self.get_binary() {
				let node = self.next().unwrap().clone();
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
		if self.next < self.list.len() {
			let mut errors = self.list[0].clone();
			let mut errors = errors.errors_mut();
			if errors.empty() {
				errors.at(self.list[self.next].span(), "expected end of expression");
			}
			return NodeEval::Complete;
		}

		if self.values.len() == 0 {
			let mut errors = self.list[0].clone();
			let mut errors = errors.errors_mut();
			if errors.empty() {
				let sta = self.list.first().and_then(|x| x.span());
				let end = self.list.last().and_then(|x| x.span());
				let span = Span::from_range(sta, end);
				errors.at(span, "invalid expression");
			}
			self.check_pending()
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

	pub fn next(&self) -> Option<&Node> {
		self.queued.front().or(self.list.get(self.next))
	}

	pub fn advance(&mut self) {
		if self.queued.pop_front().is_none() {
			self.next += 1;
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

	pub fn is_value(&mut self) -> bool {
		if let Some(node) = self.next() {
			let next = node.val();
			let expr = get_trait!(&*next, IsExprValueNode);
			if let Some(expr) = expr {
				expr.is_value()
			} else {
				false
			}
		} else {
			false
		}
	}

	pub fn get_unary_pre(&self) -> Option<OpUnary> {
		let next = self.next()?.val();
		let node = get_trait!(&*next, IsOperatorNode);
		node.and_then(|x| x.get_unary_pre())
	}

	pub fn get_binary(&self) -> Option<OpBinary> {
		let next = self.next()?.val();
		let node = get_trait!(&*next, IsOperatorNode);
		node.and_then(|x| x.get_binary())
	}

	pub fn get_ternary(&mut self) -> Option<(OpTernary, &'static str)> {
		let next = self.next()?.val();
		let node = get_trait!(&*next, IsOperatorNode);
		node.and_then(|x| x.get_ternary())
	}
}
