use std::collections::VecDeque;

use crate::{lexer::LexStream, lexer::Token, node::*, operator::*, Context, Error};

pub fn parse_indented_block(context: &mut Context) -> Node {
	let pos = context.pos();

	if !context.skip_symbol(":") {
		return Node::None(context.pos());
	}

	let ok = context.token() == Token::Break;
	let ok = if ok {
		context.advance();
		context.token() == Token::Indent
	} else {
		false
	};

	if !ok {
		let error = Error::ExpectedIndent(context.span());
		return Node::Invalid(error);
	}
	context.advance();

	let mut block = Vec::new();
	while context.token() != Token::Dedent {
		let line = parse_line(context);
		let node = match line {
			Node::Invalid(error) => {
				context.add_error(error);
				break;
			}
			Node::None(..) => {
				let error = Error::ExpectedExpression(context.next()).at("indented block");
				return Node::Invalid(error);
			}
			Node::Some(value, ..) => value,
		};
		block.push(node);
	}
	context.advance();

	let node = NodeKind::Block(block);
	Node::Some(node, context.from(pos))
}

pub fn parse_line(context: &mut Context) -> Node {
	context.scope_to_line_with_break(";");
	let node = parse_node(context);
	context.leave_scope();
	context.skip_symbol(";");
	if context.token() == Token::Break {
		context.advance();
	}
	node
}

pub fn parse_node(context: &mut Context) -> Node {
	let node = parse_expression(context);
	context.check_end();
	node
}

pub fn parse_expression(context: &mut Context) -> Node {
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
			.next()
			.symbol()
			.and_then(|next| OpUnary::get_prefix(next))
		{
			// the unary operator doesn't affect other operators on the stack
			// because it binds forward to the next operator
			let op = Op::Unary(op);
			ops.push_back(op);
			context.advance();
		}

		match parse_atom(context) {
			Node::Some(expr, ..) => {
				values.push_back(expr);
			}
			Node::None(..) => {
				return if ops.len() > 0 {
					Node::Invalid(Error::ExpectedExpression(context.next()))
				} else {
					break;
				};
			}
			Node::Invalid(span) => return Node::Invalid(span),
		};

		// TODO: posfix operators (always pop themselves)

		// Ternary and binary operators work similarly, but the ternary will
		// parse the middle expression as parenthesized.
		if let Some((op, end)) = context
			.next()
			.symbol()
			.and_then(|next| OpTernary::get(next))
		{
			context.advance();
			let op = Op::Ternary(op);
			push_op(op, &mut ops, &mut values);

			let expr = match parse_expression(context) {
				Node::Some(expr, ..) => expr,
				Node::None(..) => {
					return Node::Invalid(
						Error::ExpectedExpression(context.next()).at("ternary operator"),
					);
				}
				Node::Invalid(error) => return Node::Invalid(error),
			};
			values.push_back(expr);

			if !context.skip_symbol(end) {
				return Node::Invalid(
					Error::ExpectedSymbol(end, context.span()).at("ternary operator"),
				);
			}
		} else if let Some(op) = context.next().symbol().and_then(|next| OpBinary::get(next)) {
			let op = Op::Binary(op);
			push_op(op, &mut ops, &mut values);
			context.advance();
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

fn parse_atom(context: &mut Context) -> Node {
	let pos = context.pos();
	let value = match context.token() {
		Token::Invalid => return Node::Invalid(Error::InvalidToken(context.span())),
		Token::Identifier => {
			let value = match context.next().text() {
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
			context.advance();
			value
		}
		Token::Integer(value) => {
			context.advance();
			Atom::Integer(value).as_value()
		}
		Token::Literal(content) => {
			context.advance();
			Atom::String(content.into()).as_value()
		}
		Token::Symbol("(") => {
			context.scope_to_parenthesis();
			let next = parse_node(context);
			context.leave_scope();
			context.skip_symbol(")");
			match next {
				Node::Invalid(error) => return Node::Invalid(error),
				Node::None(..) => {
					return Node::Invalid(Error::ExpectedExpression(context.next()));
				}
				Node::Some(expr, ..) => expr,
			}
		}
		_ => return Node::None(pos),
	};
	Node::Some(value, context.from(pos))
}
