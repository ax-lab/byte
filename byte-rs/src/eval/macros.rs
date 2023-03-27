use crate::{lexer::Token, Error};

use super::{parser::parse_expression, Context, Node, NodeKind};

// TODO: if, for, print

/// Trait for low-level syntax macros operating during the parsing stage.
pub trait Macro {
	fn parse<'a>(&self, context: &mut Context<'a>) -> Option<Node<'a>>;
}

pub struct Print;

impl Macro for Print {
	fn parse<'a>(&self, context: &mut Context<'a>) -> Option<Node<'a>> {
		let pos = context.pos();

		context.next(); // skip the `print`

		let mut expr_list = Vec::new();
		let node = loop {
			if context.at_end() || context.token() == Token::Break {
				let node = NodeKind::Print(expr_list);
				break Node::Some(node, context.from(pos));
			}

			if expr_list.len() > 0 {
				if !context.skip_symbol(",") {
					let error = Error::ExpectedSymbol(",", context.span());
					break Node::Invalid(error);
				}
			}

			let expr = parse_expression(context);
			match expr {
				Node::Some(expr, ..) => {
					expr_list.push(expr);
				}
				Node::None(..) => {
					let error = Error::ExpectedExpression(context.span()).at("print");
					break Node::Invalid(error);
				}
				Node::Invalid(span) => break Node::Invalid(span),
			}
		};
		Some(node)
	}
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
