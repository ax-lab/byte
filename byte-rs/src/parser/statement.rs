use std::any::TypeId;

use crate::core::any::*;
use crate::core::input::*;
use crate::lexer::*;

#[derive(Debug)]
pub enum Statement {
	End(Cursor),
	Expr(Expr),
}

impl Statement {
	pub fn span(&self) -> Span {
		match self {
			Statement::End(pos) => Span {
				sta: pos.clone(),
				end: pos.clone(),
			},
			Statement::Expr(expr) => expr.span(),
		}
	}
}

#[derive(Debug)]
pub struct Expr {
	pub items: Vec<ExprItem>,
}

impl Expr {
	pub fn new(items: Vec<ExprItem>) -> Self {
		Self { items }
	}
	pub fn span(&self) -> Span {
		let ls = &self.items;
		let sta = ls.first().map(|x| x.span().sta).unwrap();
		let end = ls.last().map(|x| x.span().sta).unwrap();
		Span { sta, end }
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
