use crate::lexer::Context;
use crate::lexer::Token;

use super::node::*;
use super::State;

pub fn parse_node<'a>(input: &mut Context<'a>, state: &mut State) -> Node {
	let node = parse_atom(input, state);
	node
}

fn parse_atom(input: &mut Context, _state: &mut State) -> Node {
	let pos = input.pos();
	let value = match input.token() {
		Token::Invalid => NodeValue::Invalid,
		Token::Identifier => {
			let value = match input.value().text() {
				"null" => Atom::Null.as_value(),
				"true" => Atom::Bool(true).as_value(),
				"false" => Atom::Bool(false).as_value(),
				id => Atom::Id(id.into()).as_value(),
			};
			input.next();
			value
		}
		Token::Integer(value) => {
			input.next();
			Atom::Integer(value).as_value()
		}
		Token::Literal(pos, end) => {
			let content = input.source().read_text(pos, end);
			input.next();
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
	let end = input.pos();
	Node {
		pos,
		end,
		val: value,
	}
}
