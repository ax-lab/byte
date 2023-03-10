use std::collections::VecDeque;

use crate::lexer::Lex;
use crate::lexer::{Input, Span, Token};

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

pub fn parse_expression(input: Lex) -> (Lex, ExprResult) {
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

	let mut input = input;
	loop {
		while let Some(op) = input.symbol().and_then(|next| UnaryOp::get_prefix(next)) {
			// the unary operator doesn't affect other operators on the stack
			// because it binds forward to the next operator
			let op = Operator::Unary(op);
			ops.push_back(op);
			input = input.next();
		}

		input = match parse_atom(input) {
			(input, expr) => {
				match expr {
					ExprResult::Expr(expr) => {
						values.push_back(expr);
					}
					ExprResult::None => {
						return if ops.len() > 0 {
							(
								input,
								ExprResult::Error(
									input.span(),
									"expected expression after operator".into(),
								),
							)
						} else {
							break;
						};
					}
					err @ ExprResult::Error(..) => return (input, err),
				}
				input
			}
		};

		// TODO: posfix operators (always pop themselves)

		// Ternary and binary operators work similarly, but the ternary will
		// parse the middle expression as parenthesized.
		input = if let Some((op, end)) = input.symbol().and_then(|next| TernaryOp::get(next)) {
			let input = input.next();
			let op = Operator::Ternary(op);
			push_op(op, &mut ops, &mut values);

			let (input, expr) = match parse_expression(input) {
				(input, expr) => match expr {
					ExprResult::Expr(expr) => (input, expr),
					ExprResult::None => {
						return (
							input,
							ExprResult::Error(
								input.span(),
								"expected expression after ternary operator".into(),
							),
						)
					}
					err @ ExprResult::Error(..) => return (input, err),
				},
			};
			values.push_back(expr);

			let (input, ok) = input.skip_symbol(end);
			if !ok {
				return (
					input,
					ExprResult::Error(input.span(), format!("expected ternary operator '{end}'")),
				);
			}
			input
		} else if let Some(op) = input.symbol().and_then(|next| BinaryOp::get(next)) {
			let op = Operator::Binary(op);
			push_op(op, &mut ops, &mut values);
			input.next()
		} else {
			break;
		}
	}

	// pop any remaining operators on the stack.
	while ops.len() > 0 {
		pop_stack(&mut ops, &mut values);
	}

	if values.len() == 0 {
		(input, ExprResult::None)
	} else {
		assert!(values.len() == 1);
		let expr = values.pop_back().unwrap();
		(input, ExprResult::Expr(expr))
	}
}

fn parse_atom(input: Lex) -> (Lex, ExprResult) {
	let (token, span) = match input {
		Lex::Some(state) => state.pair(),
		_ => return (input, ExprResult::None),
	};

	match token {
		Token::Identifier => {
			let atom = match input.text() {
				"null" => ExprAtom::Null,
				"true" => ExprAtom::Boolean(true),
				"false" => ExprAtom::Boolean(false),
				id => ExprAtom::Var(id.into()),
			};
			(input.next(), atom.result())
		}
		Token::Integer(value) => (input.next(), ExprAtom::Integer(value).result()),
		Token::Literal(str) => {
			let content = str.content_span();
			let content = input.source().read_text(content);
			(input.next(), ExprAtom::Literal(content.into()).result())
		}
		Token::Symbol("(") => {
			let (input, expr) = parse_expression(input.next());
			let (input, expr) = match expr {
				ExprResult::Expr(expr) => {
					let (input, ok) = input.skip_symbol(")");
					if !ok {
						(
							input,
							ExprResult::Error(input.span(), "expected `)`".into()),
						)
					} else {
						(input, ExprResult::Expr(expr))
					}
				}
				ExprResult::None => (
					input,
					ExprResult::Error(input.span(), "expression expected inside '()'".into()),
				),
				err @ ExprResult::Error(..) => (input, err),
			};
			(input, expr)
		}
		_ => return (input, ExprResult::None),
	}
}
