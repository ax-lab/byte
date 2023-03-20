use crate::{lexer::Token, Error};

use super::{parser::parse_expression, Context, Expr, Node, NodeKind};

/// Trait for low-level syntax macros operating during the parsing stage.
pub trait Macro {
	fn parse<'a>(&self, context: &mut Context<'a>) -> Option<Node<'a>>;
}

pub struct Let;

impl Macro for Let {
	fn parse<'a>(&self, context: &mut Context<'a>) -> Option<Node<'a>> {
		let pos = context.pos();

		context.next(); // skip the `let` or `const`.

		let id = match context.token() {
			Token::Identifier => {
				let id = context.lex().text().to_string();
				context.next();
				id
			}
			_ => return None,
		};

		let value = if context.at_end() {
			NodeKind::Expr(Expr::Let(id, None))
		} else {
			if !context.skip_symbol("=") {
				context.add_error(Error::ExpectedSymbol("=", context.span()).at("let declaration"));
				NodeKind::Invalid
			} else {
				let expr = parse_expression(context);
				match expr.value {
					NodeKind::Expr(expr) => NodeKind::Expr(Expr::Let(id, Some(expr.into()))),
					NodeKind::None => {
						context.add_error(
							Error::ExpectedExpression(context.span()).at("let declaration"),
						);
						NodeKind::Invalid
					}
					NodeKind::Invalid => NodeKind::Invalid,
				}
			}
		};

		Some(value.at(pos, context.pos()))
	}
}
