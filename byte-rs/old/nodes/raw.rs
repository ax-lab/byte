use std::collections::VecDeque;
use std::io::Write;
use std::sync::{Arc, RwLock};

use crate::core::repr::*;
use crate::vm::operators::*;

use super::*;

/// Raw list of unprocessed atom nodes.
#[derive(Clone)]
pub struct Raw {
	parser: Option<RawExprParser>,
}

impl Raw {
	pub fn new(expr: Node) -> Node {
		let span = expr.span();
		let node = Self {
			parser: Some(RawExprParser::new(expr)),
		};
		Node::new(node).at(span)
	}

	pub fn empty() -> Self {
		Self { parser: None }
	}
}

has_traits!(Raw: IsNode, HasRepr);

impl IsNode for Raw {
	fn eval(&self, mut node: Node) -> NodeEval {
		if let Some(parser) = self.parser.as_ref() {
			parser.reduce(node)
		} else {
			node.errors_mut().add(format!("empty expression"));
			NodeEval::Complete
		}
	}
}

impl HasRepr for Raw {
	fn output_repr(&self, output: &mut Repr) -> std::io::Result<()> {
		if let Some(ref parser) = self.parser {
			write!(output, "Raw(")?;
			{
				let mut output = output.indented();
				parser.expr.output_list(&mut output)?;
			}
			write!(output, ")")?;
		} else {
			write!(output, "Raw()")?;
		}
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

impl RawExpr {
	pub fn new_unary(op: OpUnary, op_node: Node, a: Node) -> Node {
		let span = Span::from_range(op_node.span(), a.span());
		let node = RawExpr::Unary(op, a);
		Node::new(node).at(span)
	}

	pub fn new_binary(op: OpBinary, a: Node, b: Node) -> Node {
		let span = Span::from_range(a.span(), b.span());
		let node = RawExpr::Binary(op, a, b);
		Node::new(node).at(span)
	}

	pub fn new_ternary(op: OpTernary, a: Node, b: Node, c: Node) -> Node {
		let span = Span::from_range(a.span(), c.span());
		let node = RawExpr::Ternary(op, a, b, c);
		Node::new(node).at(span)
	}
}

impl IsNode for RawExpr {
	fn eval(&self, _node: Node) -> NodeEval {
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
struct RawExprParser {
	expr: Node,
	state: Arc<RwLock<RawExprParserState>>,
}

struct RawExprParserState {
	next: Option<Node>,
	ops: VecDeque<(Op, Node)>,
	values: VecDeque<Node>,
}

impl RawExprParser {
	pub fn new(expr: Node) -> Self {
		Self {
			expr: expr.clone(),
			state: Arc::new(RwLock::new(RawExprParserState {
				next: Some(expr),
				ops: Default::default(),
				values: Default::default(),
			})),
		}
	}

	pub fn reduce(&self, mut node: Node) -> NodeEval {
		let mut state = self.state.write().unwrap();
		loop {
			// Only evaluate the expression when the next node is available.
			if let Some(next) = state.next() {
				if !next.is_done() {
					return NodeEval::DependsOn(vec![next.clone()]);
				}
			}

			if let Some(op) = state.get_unary_pre() {
				// the unary operator doesn't affect other operators on the stack
				// because it binds forward to the next operator
				let op = Op::Unary(op);
				let node = state.next().clone().unwrap();
				state.push_op(op, node);
				state.advance();
				continue;
			}

			let next = state.next();
			let saved = next.clone();
			if state.is_value() {
				state.values.push_back(next.unwrap());
				state.advance();
			} else {
				let mut expr = self.expr.clone();
				let mut errors = expr.errors_mut();
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
			if let Some((op, end)) = state.get_ternary() {
				let node = state.next().unwrap();
				let op = Op::Ternary(op);
				state.push_op(op, node.clone());

				if let Some(mut node) = state.find_symbol(end) {
					let tail = node.split_next();
					node.extract();

					if let Some(..) = tail {
						// Create a raw with the sub expression and append it as a node
						let expr = Raw::new(node);
						state.values.push_back(expr);
						state.next = tail;
					} else {
						let mut expr = self.expr.clone();
						let mut errors = expr.errors_mut();
						errors.at(
							node.span(),
							format!("expected expression for ternary operator after {node}"),
						);
						return NodeEval::Complete;
					}
				} else {
					let mut expr = self.expr.clone();
					let mut errors = expr.errors_mut();
					errors.at(
						node.span(),
						format!("symbol `{end}` for ternary operator {node} not found"),
					);
					return NodeEval::Complete;
				}
			} else if let Some(op) = state.get_binary() {
				let node = state.next().unwrap().clone();
				let op = Op::Binary(op);
				state.push_op(op, node);
				state.advance();
			} else {
				break;
			}
		}

		// pop any remaining operators on the stack.
		while state.ops.len() > 0 {
			state.pop_stack();
		}

		// check that there was no unparsed portion of the expression
		if let Some(ref next) = state.next {
			let mut expr = self.expr.clone();
			let mut errors = expr.errors_mut();
			if errors.empty() {
				errors.at(next.span(), "expected end of expression");
			}
			return NodeEval::Complete;
		}

		if state.values.len() == 0 {
			let mut expr = self.expr.clone();
			let mut errors = expr.errors_mut();
			if errors.empty() {
				errors.add("invalid expression");
			}
			NodeEval::Complete
		} else {
			assert!(state.values.len() == 1);
			let expr = state.values.pop_back().unwrap();
			node.set_value_from_node(&expr);
			NodeEval::Changed
		}
	}
}

impl RawExprParserState {
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
				let expr = RawExpr::new_unary(op, op_node, expr);
				values.push_back(expr);
			}
			Op::Binary(op) => {
				let rhs = values.pop_back().unwrap();
				let lhs = values.pop_back().unwrap();
				let expr = RawExpr::new_binary(op, lhs, rhs);
				values.push_back(expr);
			}
			Op::Ternary(op) => {
				let c = values.pop_back().unwrap();
				let b = values.pop_back().unwrap();
				let a = values.pop_back().unwrap();
				let expr = RawExpr::new_ternary(op, a, b, c);
				values.push_back(expr);
			}
		}
	}

	//------------------------------------------------------------------------//
	// Helpers
	//------------------------------------------------------------------------//

	pub fn next(&self) -> Option<Node> {
		self.next.clone()
	}

	pub fn advance(&mut self) {
		self.next = self.next.as_ref().and_then(|x| x.next())
	}

	pub fn find_symbol(&self, symbol: &str) -> Option<Node> {
		let mut next = self.next();
		while let Some(node) = next {
			if let Some(atom) = node.get::<Atom>() {
				if atom.symbol() == Some(symbol) {
					return Some(node.clone());
				}
			}
			next = node.next();
		}
		None
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
