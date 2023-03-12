use std::collections::VecDeque;

use crate::lexer::Span;
use crate::lexer::{Context, Token};

use super::operators::*;

#[derive(Debug)]
pub enum ExprAtom {
	Var(String),
	Integer(u64),
	Literal(String),
	Boolean(bool),
	Null,
}

#[derive(Debug)]
pub enum ExprResult<'a> {
	None,
	Error(Span<'a>, String),
	Expr(Expr),
}

impl<'a> From<Expr> for ExprResult<'a> {
	fn from(expr: Expr) -> Self {
		ExprResult::Expr(expr)
	}
}

impl<'a> From<ExprAtom> for ExprResult<'a> {
	fn from(atom: ExprAtom) -> Self {
		ExprResult::Expr(Expr::Value(atom))
	}
}

#[derive(Debug)]
pub enum Expr {
	Value(ExprAtom),
	Unary(UnaryOp, Box<Expr>),
	Binary(BinaryOp, Box<Expr>, Box<Expr>),
	Ternary(TernaryOp, Box<Expr>, Box<Expr>, Box<Expr>),
}

pub trait AsResult<'a> {
	fn result(self) -> ExprResult<'a>;
}

impl<'a, T: Into<ExprResult<'a>>> AsResult<'a> for T {
	fn result(self) -> ExprResult<'a> {
		self.into()
	}
}

pub fn parse_expression<'a>(input: &mut Context<'a>) -> ExprResult<'a> {
	let mut ops = VecDeque::new();
	let mut values = VecDeque::new();

	// pop a single operation from the stack
	let pop_stack = |ops: &mut VecDeque<Operator>, values: &mut VecDeque<Expr>| {
		let op = ops.pop_back().unwrap();
		match op {
			Operator::Unary(op) => {
				let expr = values.pop_back().unwrap();
				let expr = Expr::Unary(op, expr.into());
				values.push_back(expr);
			}
			Operator::Binary(op) => {
				let rhs = values.pop_back().unwrap();
				let lhs = values.pop_back().unwrap();
				let expr = Expr::Binary(op, lhs.into(), rhs.into());
				values.push_back(expr);
			}
			Operator::Ternary(op) => {
				let c = values.pop_back().unwrap();
				let b = values.pop_back().unwrap();
				let a = values.pop_back().unwrap();
				let expr = Expr::Ternary(op, a.into(), b.into(), c.into());
				values.push_back(expr);
			}
			op => todo!("{op:?}"),
		}
	};

	// push an operator onto the stack, popping operations with higher precedence
	let push_op = |op: Operator, ops: &mut VecDeque<Operator>, values: &mut VecDeque<Expr>| {
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
		while let Some(op) = input
			.value
			.symbol()
			.and_then(|next| UnaryOp::get_prefix(next))
		{
			// the unary operator doesn't affect other operators on the stack
			// because it binds forward to the next operator
			let op = Operator::Unary(op);
			ops.push_back(op);
			input.next();
		}

		match parse_atom(input) {
			ExprResult::Expr(expr) => {
				values.push_back(expr);
			}
			ExprResult::None => {
				return if ops.len() > 0 {
					ExprResult::Error(input.span(), "expected expression after operator".into())
				} else {
					break;
				};
			}
			err @ ExprResult::Error(..) => return err,
		};

		// TODO: posfix operators (always pop themselves)

		// Ternary and binary operators work similarly, but the ternary will
		// parse the middle expression as parenthesized.
		if let Some((op, end)) = input.value.symbol().and_then(|next| TernaryOp::get(next)) {
			input.next();
			let op = Operator::Ternary(op);
			push_op(op, &mut ops, &mut values);

			let expr = match parse_expression(input) {
				ExprResult::Expr(expr) => expr,
				ExprResult::None => {
					return ExprResult::Error(
						input.span(),
						"expected expression after ternary operator".into(),
					);
				}
				err @ ExprResult::Error(..) => return err,
			};
			values.push_back(expr);

			if !input.skip_symbol(end) {
				return ExprResult::Error(
					input.span(),
					format!("expected ternary operator '{end}'"),
				);
			}
		} else if let Some(op) = input.value.symbol().and_then(|next| BinaryOp::get(next)) {
			let op = Operator::Binary(op);
			push_op(op, &mut ops, &mut values);
			input.next();
		} else {
			break;
		}
	}

	// pop any remaining operators on the stack.
	while ops.len() > 0 {
		pop_stack(&mut ops, &mut values);
	}

	if values.len() == 0 {
		ExprResult::None
	} else {
		assert!(values.len() == 1);
		let expr = values.pop_back().unwrap();
		ExprResult::Expr(expr)
	}
}

fn parse_atom<'a>(input: &mut Context<'a>) -> ExprResult<'a> {
	match input.token() {
		Token::Identifier => {
			let atom = match input.value.text() {
				"null" => ExprAtom::Null,
				"true" => ExprAtom::Boolean(true),
				"false" => ExprAtom::Boolean(false),
				id => ExprAtom::Var(id.into()),
			};
			input.next();
			atom.result()
		}
		Token::Integer(value) => {
			input.next();
			ExprAtom::Integer(value).result()
		}
		Token::Literal(content) => {
			let content = input.source().read_text(content.pos, content.end);
			input.next();
			ExprAtom::Literal(content.into()).result()
		}
		Token::Symbol("(") => {
			input.next();
			match parse_expression(input) {
				ExprResult::Expr(expr) => {
					if !input.skip_symbol(")") {
						ExprResult::Error(input.span(), "expected `)`".into())
					} else {
						ExprResult::Expr(expr)
					}
				}
				ExprResult::None => {
					ExprResult::Error(input.span(), "expression expected inside '()'".into())
				}
				err @ ExprResult::Error(..) => err,
			}
		}
		_ => return ExprResult::None,
	}
}
