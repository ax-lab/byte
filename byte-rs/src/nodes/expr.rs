use std::collections::VecDeque;

use crate::core::error::*;
use crate::core::input::*;
use crate::lang::operator::*;
use crate::lexer::*;
use crate::parser::*;

use super::*;

#[derive(Debug)]
pub enum Expr {
	Value(ExprValue),
	Unary(OpUnary, Box<Expr>),
	Binary(OpBinary, Box<Expr>, Box<Expr>),
	Ternary(OpTernary, Box<Expr>, Box<Expr>, Box<Expr>),
}

impl IsNode for Expr {}

pub fn parse_expression(ctx: &mut Context, input: &[ExprItem]) -> Option<Node> {
	let sta = input.first().unwrap().span().sta;
	let end = input.last().unwrap().span().end;
	let span = Span { sta, end };

	let expr = {
		let mut iter = ExprIter { ctx, items: input };
		parse(&mut iter)
	};
	if let Some(expr) = expr {
		Some(Node::new(expr, span))
	} else {
		if !ctx.has_errors() {
			ctx.add_error(Error::new(span, ParserError::InvalidExpression));
		}
		None
	}
}

fn parse(input: &mut ExprIter) -> Option<Expr> {
	let mut ops = VecDeque::new();
	let mut values = VecDeque::new();

	// pop a single operation from the stack
	let pop_stack = |ops: &mut VecDeque<Op>, values: &mut VecDeque<Expr>| {
		let op = ops.pop_back().unwrap();
		match op {
			Op::Unary(op) => {
				let expr = values.pop_back().unwrap();
				let expr = Expr::Unary(op, expr.into());
				values.push_back(expr);
			}
			Op::Binary(op) => {
				let rhs = values.pop_back().unwrap();
				let lhs = values.pop_back().unwrap();
				let expr = Expr::Binary(op, lhs.into(), rhs.into());
				values.push_back(expr);
			}
			Op::Ternary(op) => {
				let c = values.pop_back().unwrap();
				let b = values.pop_back().unwrap();
				let a = values.pop_back().unwrap();
				let expr = Expr::Ternary(op, a.into(), b.into(), c.into());
				values.push_back(expr);
			}
		}
	};

	// push an operator onto the stack, popping operations with higher precedence
	let push_op = |op: Op, ops: &mut VecDeque<Op>, values: &mut VecDeque<Expr>| {
		while let Some(top) = ops.back() {
			if top < &op {
				pop_stack(ops, values);
			} else {
				break;
			}
		}
		ops.push_back(op);
	};

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

pub struct ExprIter<'a> {
	ctx: &'a mut Context,
	items: &'a [ExprItem],
}

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
