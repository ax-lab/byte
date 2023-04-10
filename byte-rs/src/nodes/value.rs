use crate::lang::*;
use crate::lexer::*;

use super::*;

#[allow(unused)]
#[derive(Debug)]
pub enum ExprValue {
	Bool(bool),
	Id(String),
	Integer(u64),
	Literal(String),
}

impl IsNode for ExprValue {}

pub fn parse_value(input: &mut ExprIter) -> Option<Expr> {
	let value = match input.get_token() {
		Some(next) => match next.token() {
			Token::Invalid => return None,
			Token::Identifier => {
				let value = match next.text() {
					"true" => ExprValue::Bool(true),
					"false" => ExprValue::Bool(false),
					id => {
						if let Some(expr) = input.parse_macro(id) {
							return Some(expr);
						}
						ExprValue::Id(id.into())
					}
				};
				input.advance();
				value
			}
			token @ Token::Other(..) => {
				if let Some(value) = token.get::<Integer>() {
					input.advance();
					ExprValue::Integer(*value)
				} else if let Some(value) = token.get::<Literal>() {
					input.advance();
					ExprValue::Literal(value.clone())
				} else {
					return None;
				}
			}
			_ => return None,
		},
		_ => return None,
	};
	Some(Expr::Value(value))
}
