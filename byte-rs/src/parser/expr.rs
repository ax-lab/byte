use std::collections::VecDeque;

use crate::lexer::ReadToken;
use crate::lexer::{Input, Span, Token, TokenStream};

use super::operators::*;
use super::ParseResult;

#[derive(Debug)]
pub enum ExprAtom {
	Var(String),
	Integer(u64),
	Literal(String),
	Boolean(bool),
	Null,
}

#[derive(Debug)]
pub enum ExprResult {
	None,
	Error(Span, String),
	Expr(Expr),
}

impl From<Expr> for ExprResult {
	fn from(expr: Expr) -> Self {
		ExprResult::Expr(expr)
	}
}

impl From<ExprAtom> for ExprResult {
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
	List(ListOp, Vec<Expr>),
}

pub trait AsResult {
	fn result(self) -> ExprResult;
}

impl<T: Into<ExprResult>> AsResult for T {
	fn result(self) -> ExprResult {
		self.into()
	}
}

pub fn parse_expression<T: Input>(input: &mut TokenStream<T>) -> ExprResult {
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
		while let Some(op) = input.map_symbol(|next| UnaryOp::get_prefix(next)) {
			// the unary operator doesn't affect other operators on the stack
			// because it binds forward to the next operator
			let op = Operator::Unary(op);
			ops.push_back(op);
		}

		match parse_atom(input) {
			ExprResult::Expr(expr) => {
				values.push_back(expr);
			}
			ExprResult::None => {
				return if ops.len() > 0 {
					ExprResult::Error(
						input.next_span(),
						"expected expression after operator".into(),
					)
				} else {
					break;
				};
			}
			err @ ExprResult::Error(..) => return err,
		}

		// TODO: posfix operators (always pop themselves)

		// Ternary and binary operators work similarly, but the ternary will
		// parse the middle expression as parenthesized.
		if let Some((op, end)) = input.map_symbol(|next| TernaryOp::get(next)) {
			let op = Operator::Ternary(op);
			push_op(op, &mut ops, &mut values);

			let expr = match parse_expression(input) {
				ExprResult::Expr(expr) => expr,
				ExprResult::None => {
					return ExprResult::Error(
						input.next_span(),
						"expected expression after ternary operator".into(),
					)
				}
				err @ ExprResult::Error(..) => return err,
			};
			values.push_back(expr);

			if let Some(error) = input.expect_symbol(end, |span| {
				ExprResult::Error(span, format!("expected ternary operator '{end}'"))
			}) {
				return error;
			}
		} else if let Some(op) = input.map_symbol(|next| BinaryOp::get(next)) {
			let op = Operator::Binary(op);
			push_op(op, &mut ops, &mut values);
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

fn parse_atom<T: Input>(input: &mut TokenStream<T>) -> ExprResult {
	input
		.try_read(|input, token, span| {
			let result = match token {
				Token::Identifier(id) => {
					let atom = match id.as_str() {
						"null" => ExprAtom::Null,
						"true" => ExprAtom::Boolean(true),
						"false" => ExprAtom::Boolean(false),
						_ => ExprAtom::Var(id),
					};
					atom.result()
				}
				Token::Integer(value) => ExprAtom::Integer(value).result(),
				Token::Literal(text) => ExprAtom::Literal(text).result(),
				Token::Symbol("(") => {
					let expr = parse_expression(input);
					match expr {
						ExprResult::Expr(expr) => {
							if let Some(error) = input.expect_symbol(")", |span| {
								ExprResult::Error(span, "expected ')'".into())
							}) {
								error
							} else {
								ExprResult::Expr(expr)
							}
						}
						ExprResult::None => ExprResult::Error(
							input.next_span(),
							"expression expected inside '()'".into(),
						),
						err @ ExprResult::Error(..) => err,
					}
				}
				_ => return ReadToken::Unget(token),
			};
			ReadToken::MapTo(result)
		})
		.unwrap_or(ExprResult::None)
}
