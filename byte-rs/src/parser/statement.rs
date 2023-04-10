use std::any::TypeId;

use crate::core::any::*;
use crate::core::input::*;
use crate::lexer::*;
use crate::nodes::*;

use super::*;

#[derive(Debug)]
pub enum Statement {
	End(Cursor),
	Expr(Vec<ExprItem>),
}

impl Statement {
	pub fn resolve(&self, ctx: &mut Context) -> Option<Node> {
		match self {
			Statement::End(..) => None,
			Statement::Expr(expr) => crate::nodes::parse_expression(ctx, &expr),
		}
	}
}

#[allow(unused)]
#[derive(Debug)]
pub enum ExprItem {
	Token(TokenAt),
	Parenthesized {
		node: Statement,
		start: TokenAt,
		end: TokenAt,
	},
	Macro {
		value: MacroValue,
		span: Span,
	},
}

impl ExprItem {
	pub fn span(&self) -> Span {
		match self {
			ExprItem::Token(token) => token.span(),
			ExprItem::Parenthesized { start, end, .. } => {
				let sta = start.span().sta;
				let end = end.span().end;
				Span { sta, end }
			}
			ExprItem::Macro { span, .. } => span.clone(),
		}
	}
}

#[allow(unused)]
pub struct MacroValue {
	name: &'static str,
	kind: TypeId,
	value: Value,
}

impl std::fmt::Debug for MacroValue {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "<{}: ", self.name)?;
		self.value.fmt(f)?;
		write!(f, ">")
	}
}
