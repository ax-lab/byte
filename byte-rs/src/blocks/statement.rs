use crate::lexer::*;

use super::*;

#[derive(Clone)]
pub struct Statement {
	expr: Vec<TokenAt>,
}

impl Statement {
	pub fn print(&self) {
		print!(
			"{:04}| ",
			self.expr
				.first()
				.map(|x| x.span().sta.row() + 1)
				.unwrap_or_default()
		);
		for (i, it) in self.expr.iter().enumerate() {
			if i > 0 {
				print!(", ");
			}
			print!("{it}");
		}
		println!();
	}
}

impl IsBlock for Statement {
	type Value = Self;

	fn parse<T: Scope>(mut scope: T) -> (T, Self::Value) {
		let mut expr = Vec::new();
		loop {
			let next = scope.read();
			if !next.is_some() {
				break;
			}
			expr.push(next);
		}
		(scope, Statement { expr })
	}
}

#[derive(Clone)]
pub struct StatementScope<T: Scope> {
	inner: T,
}

impl<T: Scope> StatementScope<T> {
	pub fn new(inner: T) -> Self {
		Self { inner }
	}

	pub fn into_inner(self) -> T {
		self.inner
	}
}

impl<T: Scope> Scope for StatementScope<T> {
	fn read(&mut self) -> crate::lexer::TokenAt {
		let next = self.inner.read();
		if next.token() == Token::Break {
			next.as_none()
		} else {
			next
		}
	}
}
