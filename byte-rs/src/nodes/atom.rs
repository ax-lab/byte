use crate::core::str::*;
use crate::lang::*;
use crate::lexer::*;

use super::*;

#[derive(Debug)]
pub struct Atom(TokenAt);

impl Atom {
	fn value(&self) -> &TokenAt {
		&self.0
	}
}

#[cfg(never)]
impl IsNode for Atom {
	fn is_value(&self) -> Option<bool> {
		let result = match self.value().token() {
			Token::Identifier => true,
			token @ Token::Other(..) => token.is::<Integer>() || token.is::<Literal>(),
			_ => false,
		};
		Some(result)
	}

	fn resolve(&self, scope: &mut Scope, errors: &mut ErrorList) -> Option<Expr> {
		let value = self.value();
		let expr = match value.token() {
			Token::Identifier => {
				let expr = match value.text() {
					"true" => Expr::new(expr::Literal::Bool(true)),
					"false" => Expr::new(expr::Literal::Bool(false)),
					id => todo!(),
				};
				Some(expr)
			}
			token @ Token::Other(..) => {
				if let Some(value) = token.get::<Integer>() {
					let expr = Expr::new(expr::Literal::Integer(*value));
					Some(expr)
				} else if let Some(value) = token.get::<Literal>() {
					let expr = Expr::new(expr::Literal::String(Str::from(value.clone())));
					Some(expr)
				} else {
					None
				}
			}
			_ => None,
		};
		let span = value.span();
		if let Some(expr) = expr {
			let expr = expr.at(span);
			Some(expr)
		} else {
			errors.at(span, format!("{value} is not a value"));
			None
		}
	}
}
