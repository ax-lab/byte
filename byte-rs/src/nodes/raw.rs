use std::collections::VecDeque;

use crate::core::input::*;
use crate::lexer::*;
use crate::vm::operators::*;

use super::*;

#[derive(Debug)]
pub enum RawExpr {
	Unary(OpUnary, Node),
	Binary(OpBinary, Node, Node),
	Ternary(OpTernary, Node, Node, Node),
}

#[cfg(never)]
impl IsNode for RawExpr {
	fn is_value(&self) -> Option<bool> {
		Some(true)
	}

	fn resolve(&self, scope: &mut Scope, errors: &mut ErrorList) -> Option<Expr> {
		todo!()
	}
}

#[cfg(never)]
#[derive(Debug)]
pub struct Raw {
	list: Vec<Node>,
}

#[cfg(never)]
impl IsNode for Raw {
	fn is_value(&self) -> Option<bool> {
		Some(true)
	}

	fn resolve(&self, scope: &mut Scope, errors: &mut ErrorList) -> Option<Expr> {
		// if let Some(expr) = parse(&self.list, scope, errors) {
		// 	expr.complete(scope, errors)
		// } else {
		// 	None
		// }
		todo!()
	}
}

//----------------------------------------------------------------------------//
// Expression parsing
//----------------------------------------------------------------------------//

/// Provides incremental expression parsing for a list of [`Node`].
#[cfg(never)]
struct RawParser {
	next: VecDeque<Node>,
	ops: VecDeque<(Op, Node)>,
	values: VecDeque<Node>,
}

#[cfg(never)]
impl RawParser {
	pub fn is_complete(&self) -> bool {
		self.next.len() == 0
	}

	pub fn reduce(&mut self, errors: &mut ErrorList) -> bool {
		loop {
			let next = if let Some(pending) = self.next.front().map(|x| x.is_pending()) {
				if pending {
					// we can't process this node yet
					return false;
				}
				self.next.pop_front().unwrap()
			} else {
				// the whole expression has been parsed
				return true;
			};
		}
		todo!()
	}

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
				let expr = Node::new(expr).set_span(span);
				values.push_back(expr);
			}
			Op::Binary(op) => {
				let rhs = values.pop_back().unwrap();
				let lhs = values.pop_back().unwrap();
				let span = Node::get_span(&lhs, &rhs);
				let expr = RawExpr::Binary(op, lhs, rhs);
				let expr = Node::new(expr).set_span(span);
				values.push_back(expr);
			}
			Op::Ternary(op) => {
				let c = values.pop_back().unwrap();
				let b = values.pop_back().unwrap();
				let a = values.pop_back().unwrap();
				let span = Node::get_span(&a, &c);
				let expr = RawExpr::Ternary(op, a, b, c);
				let expr = Node::new(expr).set_span(span);
				values.push_back(expr);
			}
		}
	}
}

#[cfg(never)]
fn parse(input: &[Node], scope: &mut Scope, errors: &mut ErrorList) -> Option<Node> {
	let mut ops = VecDeque::new();
	let mut values = VecDeque::new();

	// pop a single operation from the stack
	let pop_stack = |ops: &mut VecDeque<Op>, values: &mut VecDeque<Expr>| {};

	// push an operator onto the stack, popping operations with higher precedence
	let push_op = |op: Op, ops: &mut VecDeque<Op>, values: &mut VecDeque<Expr>| {};

	loop {
		while let Some(op) = input.get_unary() {
			// the unary operator doesn't affect other operators on the stack
			// because it binds forward to the next operator
			let op = Op::Unary(op);
			ops.push_back(op);
			input.advance();
		}

		if let Some(expr) = input.parse_atom() {
			values.push_back(expr);
		} else {
			if ops.len() > 0 {
				input.add_error(input.span(), ParserError::ExpectedExpressionValue);
				return None;
			} else {
				break;
			};
		}

		// TODO: posfix operators (always pop themselves)

		// Ternary and binary operators work similarly, but the ternary will
		// parse the middle expression as parenthesized.
		if let Some((op, end)) = input.get_ternary() {
			input.advance();
			let op = Op::Ternary(op);
			push_op(op, &mut ops, &mut values);

			let expr = match parse(input) {
				Some(expr) => expr,
				None => {
					input.add_error(input.span(), ParserError::ExpectedExpressionValue);
					return None;
				}
			};
			values.push_back(expr);

			if !input.skip_symbol(end) {
				input.add_error(
					input.span(),
					ParserError::ExpectedSymbol {
						symbol: end,
						context: "ternary operator",
					},
				);
				return None;
			}
		} else if let Some(op) = input.get_binary() {
			let op = Op::Binary(op);
			push_op(op, &mut ops, &mut values);
			input.advance();
		} else {
			break;
		}
	}

	// pop any remaining operators on the stack.
	while ops.len() > 0 {
		pop_stack(&mut ops, &mut values);
	}

	if values.len() == 0 {
		None
	} else {
		assert!(values.len() == 1);
		let expr = values.pop_back().unwrap();
		Some(expr)
	}
}

#[cfg(never)]
pub struct ExprIter<'a> {
	ctx: &'a mut Context,
	items: &'a [ExprItem],
}

#[cfg(never)]
impl<'a> ExprIter<'a> {
	pub fn span(&self) -> Span {
		self.items.first().unwrap().span()
	}

	pub fn skip_symbol(&mut self, expected: &'static str) -> bool {
		if let Some(token) = self.get_token() {
			if token.symbol() == Some(expected) {
				self.advance();
				true
			} else {
				false
			}
		} else {
			false
		}
	}

	pub fn get_unary(&self) -> Option<OpUnary> {
		if let Some(next) = self.get_token() {
			if let Some(symbol) = next.symbol() {
				OpUnary::get_prefix(symbol)
			} else {
				None
			}
		} else {
			None
		}
	}

	pub fn get_binary(&self) -> Option<OpBinary> {
		if let Some(next) = self.get_token() {
			if let Some(symbol) = next.symbol() {
				OpBinary::get(symbol)
			} else {
				None
			}
		} else {
			None
		}
	}

	pub fn get_ternary(&self) -> Option<(OpTernary, &'static str)> {
		if let Some(next) = self.get_token() {
			if let Some(symbol) = next.symbol() {
				OpTernary::get(symbol)
			} else {
				None
			}
		} else {
			None
		}
	}

	pub fn parse_atom(&mut self) -> Option<Expr> {
		if self.items.first().is_some() {
			parse_value(self)
		} else {
			None
		}
	}

	pub fn parse_macro(&mut self, name: &str) -> Option<Expr> {
		if name == "print" {
			todo!()
		} else {
			None
		}
	}

	pub fn advance(&mut self) {
		self.items = &self.items[1..];
	}

	pub fn get_token(&self) -> Option<TokenAt> {
		if let Some(next) = self.items.first() {
			match next {
				ExprItem::Token(token) => Some(token.clone()),
				_ => None,
			}
		} else {
			None
		}
	}

	pub fn add_error<T: IsError>(&mut self, span: Span, error: T) {
		self.ctx.add_error(Error::new(span, error));
	}
}
