use crate::lexer::Token;

use super::node::*;
use super::Context;

pub fn parse_node<'a>(context: &mut Context<'a>) -> Node<'a> {
	let node = parse_atom(context);
	context.check_end();
	node
}

fn parse_atom<'a>(context: &mut Context<'a>) -> Node<'a> {
	let pos = context.pos();
	let value = match context.token() {
		Token::Invalid => NodeValue::Invalid,
		Token::Identifier => {
			let value = match context.lex().text() {
				"null" => Atom::Null.as_value(),
				"true" => Atom::Bool(true).as_value(),
				"false" => Atom::Bool(false).as_value(),
				id => Atom::Id(id.into()).as_value(),
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
		// Token::Symbol("(") => {
		// 	input.next();
		// 	match parse_node(input, state) {
		// 		ExprResult::Expr(expr) => {
		// 			if !input.skip_symbol(")") {
		// 				ExprResult::Error(input.span(), "expected `)`".into())
		// 			} else {
		// 				ExprResult::Expr(expr)
		// 			}
		// 		}
		// 		ExprResult::None => {
		// 			ExprResult::Error(input.span(), "expression expected inside '()'".into())
		// 		}
		// 		err @ ExprResult::Error(..) => err,
		// 	}
		// }
		_ => NodeValue::None,
	};
	let end = context.pos();
	Node::new(pos, end, value)
}
