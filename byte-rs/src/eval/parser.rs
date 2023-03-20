use std::collections::VecDeque;

use crate::lexer::Token;
use crate::Error;

use super::node::*;
use super::Context;
use super::Op;
use super::OpBinary;
use super::OpTernary;
use super::OpUnary;

pub fn parse_node<'a>(context: &mut Context<'a>) -> Node<'a> {
	let node = parse_expression(context);
	context.check_end();
	node
}

pub fn parse_expression<'a>(context: &mut Context<'a>) -> Node<'a> {
	let pos = context.pos();
	let mut ops = VecDeque::new();
	let mut values = VecDeque::new();

	// pop a single operation from the stack
	let pop_stack = |ops: &mut VecDeque<Op>, values: &mut VecDeque<NodeKind>| {
		let op = ops.pop_back().unwrap();
		match op {
			Op::Unary(op) => {
				let expr = values.pop_back().unwrap();
				let expr = NodeKind::Unary(op, expr.into());
				values.push_back(expr);
			}
			Op::Binary(op) => {
				let rhs = values.pop_back().unwrap();
				let lhs = values.pop_back().unwrap();
				let expr = NodeKind::Binary(op, lhs.into(), rhs.into());
				values.push_back(expr);
			}
			Op::Ternary(op) => {
				let c = values.pop_back().unwrap();
				let b = values.pop_back().unwrap();
				let a = values.pop_back().unwrap();
				let expr = NodeKind::Ternary(op, a.into(), b.into(), c.into());
				values.push_back(expr);
			}
		}
	};

	// push an operator onto the stack, popping operations with higher precedence
	let push_op = |op: Op, ops: &mut VecDeque<Op>, values: &mut VecDeque<NodeKind>| {
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
		while let Some(op) = context
			.lex()
			.symbol()
			.and_then(|next| OpUnary::get_prefix(next))
		{
			// the unary operator doesn't affect other operators on the stack
			// because it binds forward to the next operator
			let op = Op::Unary(op);
			ops.push_back(op);
			context.next();
		}

		match parse_atom(context) {
			Node::Some(expr, ..) => {
				values.push_back(expr);
			}
			Node::None(..) => {
				return if ops.len() > 0 {
					context.add_error(Error::ExpectedExpression(context.span()));
					Node::Invalid(context.from(pos))
				} else {
					break;
				};
			}
			Node::Invalid(span) => return Node::Invalid(span),
		};

		// TODO: posfix operators (always pop themselves)

		// Ternary and binary operators work similarly, but the ternary will
		// parse the middle expression as parenthesized.
		if let Some((op, end)) = context.lex().symbol().and_then(|next| OpTernary::get(next)) {
			context.next();
			let op = Op::Ternary(op);
			push_op(op, &mut ops, &mut values);

			let expr = match parse_expression(context) {
				Node::Some(expr, ..) => expr,
				Node::None(..) => {
					context.add_error(
						Error::ExpectedExpression(context.span()).at("ternary operator"),
					);
					return Node::Invalid(context.from(pos));
				}
				Node::Invalid(..) => return Node::Invalid(context.from(pos)),
			};
			values.push_back(expr);

			if !context.skip_symbol(end) {
				context
					.add_error(Error::ExpectedSymbol(end, context.span()).at("ternary operator"));
				return Node::Invalid(context.from(pos));
			}
		} else if let Some(op) = context.lex().symbol().and_then(|next| OpBinary::get(next)) {
			let op = Op::Binary(op);
			push_op(op, &mut ops, &mut values);
			context.next();
		} else {
			break;
		}
	}

	// pop any remaining operators on the stack.
	while ops.len() > 0 {
		pop_stack(&mut ops, &mut values);
	}

	if values.len() == 0 {
		Node::None(pos)
	} else {
		assert!(values.len() == 1);
		let expr = values.pop_back().unwrap();
		Node::Some(expr, context.from(pos))
	}
}

fn parse_atom<'a>(context: &mut Context<'a>) -> Node<'a> {
	let pos = context.pos();
	let value = match context.token() {
		Token::Invalid => return Node::Invalid(context.span()),
		Token::Identifier => {
			let value = match context.lex().text() {
				"null" => Atom::Null.as_value(),
				"true" => Atom::Bool(true).as_value(),
				"false" => Atom::Bool(false).as_value(),
				id => {
					let saved = context.clone();
					if let Some(parser) = context.get_macro(id) {
						if let Some(result) = parser.parse(context) {
							return result;
						}
					}
					*context = saved;
					Atom::Id(id.into()).as_value()
				}
			};
			context.next();
			value
		}
		Token::Integer(value) => {
			context.next();
			Atom::Integer(value).as_value()
		}
		Token::Literal(pos, end) => {
			let content = context.source().read_text(pos, end);
			context.next();
			Atom::String(content.into()).as_value()
		}
		Token::Symbol("(") => {
			*context = context.clone().scope_parenthesized("(", ")");
			let next = parse_node(context);
			*context = context.clone().pop_scope();
			match next {
				Node::Invalid(..) => return next,
				Node::None(..) => {
					if context.is_valid() {
						context.add_error(Error::ExpectedExpression(context.span()));
					}
					return Node::Invalid(context.from(pos));
				}
				Node::Some(expr, ..) => expr,
			}
		}
		_ => return Node::None(pos),
	};
	Node::Some(value, context.from(pos))
}
