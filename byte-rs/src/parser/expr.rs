use std::collections::VecDeque;

use crate::lexer::{Reader, Span, Token, TokenStream};

use super::operators::*;
use super::ParseResult;

#[derive(Debug)]
pub enum ExprAtom {
	Var(String),
	Integer(String),
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

#[derive(Debug)]
pub enum Expr {
	Value(ExprAtom),
	Unary(UnaryOp, Box<Expr>),
	Binary(BinaryOp, Box<Expr>, Box<Expr>),
	Ternary(TernaryOp, Box<Expr>, Box<Expr>, Box<Expr>),
	List(ListOp, Vec<Expr>),
}

pub fn parse_expression<T: Reader>(input: &mut TokenStream<T>) -> ExprResult {
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
		let mut next = input.text();
		while let Some(op) = UnaryOp::get_prefix(next) {
			// the unary operator doesn't affect other operators on the stack
			// because it binds forward to the next operator
			let op = Operator::Unary(op);
			ops.push_back(op);

			// move lexer forward
			input.shift();
			next = input.text();
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
		}

		// TODO: posfix operators (always pop themselves)

		// Ternary and binary operators work similarly, but the ternary will
		// parse the middle expression as parenthesized.
		let next = input.text();
		if let Some((op, end)) = TernaryOp::get(next) {
			let op = Operator::Ternary(op);
			push_op(op, &mut ops, &mut values);
			input.shift();

			let expr = match parse_expression(input) {
				ExprResult::Expr(expr) => expr,
				ExprResult::None => {
					return ExprResult::Error(
						input.span(),
						"expected expression after ternary operator".into(),
					)
				}
				err @ ExprResult::Error(..) => return err,
			};
			values.push_back(expr);

			if input.text() != end {
				return ExprResult::Error(
					input.span(),
					format!("expected ternary operator '{end}'"),
				);
			}
			input.shift();
		} else if let Some(op) = BinaryOp::get(next) {
			let op = Operator::Binary(op);
			push_op(op, &mut ops, &mut values);
			input.shift();
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

fn parse_atom<T: Reader>(input: &mut TokenStream<T>) -> ExprResult {
	let value = match input.get() {
		Token::Identifier => {
			let text = input.text();
			match text {
				"null" => ExprAtom::Null,
				"true" => ExprAtom::Boolean(true),
				"false" => ExprAtom::Boolean(false),
				id => ExprAtom::Var(id.into()),
			}
		}
		Token::Integer => ExprAtom::Integer(input.text().into()),
		Token::String => {
			let text = input.text();
			let text = text.strip_prefix("'").unwrap();
			let text = text.strip_suffix("'").unwrap();
			ExprAtom::Literal(text.into())
		}
		Token::Symbol => {
			let next = input.text();
			if next == "(" {
				input.shift();
				let expr = parse_expression(input);
				match expr {
					ExprResult::Expr(expr) => {
						let expr = if input.text() == ")" {
							input.shift();
							ExprResult::Expr(expr)
						} else {
							ExprResult::Error(input.span(), "expected ')'".into())
						};
						return expr;
					}
					ExprResult::None => {
						return ExprResult::Error(
							input.span(),
							"expression expected inside '()'".into(),
						)
					}
					err @ ExprResult::Error(..) => return err,
				}
			}
			return ExprResult::None;
		}
		_ => return ExprResult::None,
	};

	input.shift();
	ExprResult::Expr(Expr::Value(value))
}
