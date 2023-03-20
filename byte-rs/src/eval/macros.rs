use crate::{lexer::Token, Error};

use super::{parser::parse_expression, Context, Node, NodeKind};

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
			Node::Some(NodeKind::Let(id, None), context.from(pos))
		} else {
			if !context.skip_symbol("=") {
				Node::Invalid(Error::ExpectedSymbol("=", context.span()).at("let declaration"))
			} else {
				let expr = parse_expression(context);
				match expr {
					Node::Some(expr, ..) => {
						Node::Some(NodeKind::Let(id, Some(expr.into())), context.from(pos))
					}
					Node::None(..) => Node::Invalid(
						Error::ExpectedExpression(context.span()).at("let declaration"),
					),
					Node::Invalid(span) => Node::Invalid(span),
				}
			}
		};

		Some(value)
	}
}
