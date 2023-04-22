use crate::core::str::*;
use crate::lang::*;
use crate::lexer::*;
use crate::vm::operators::*;

use super::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Atom(TokenAt);

impl From<TokenAt> for Atom {
	fn from(value: TokenAt) -> Self {
		Atom(value)
	}
}

has_traits!(Atom: IsNode, IsExprValueNode, IsOperatorNode);

impl IsNode for Atom {
	fn eval(&mut self, errors: &mut ErrorList) -> NodeEval {
		NodeEval::Complete
	}

	fn span(&self) -> Option<Span> {
		Some(self.0.span())
	}
}

impl std::fmt::Display for Atom {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let Atom(value) = self;
		write!(f, "{value}")
	}
}

impl IsExprValueNode for Atom {
	fn is_value(&self) -> Option<bool> {
		let Atom(value) = self;
		match value.token() {
			Token::Identifier => Some(true),
			token @ Token::Other(..) => Some(token.is::<Integer>() || token.is::<Literal>()),
			_ => Some(false),
		}
	}
}

impl IsOperatorNode for Atom {
	fn get_unary_pre(&self) -> Option<OpUnary> {
		if let Some(symbol) = self.symbol() {
			OpUnary::get_prefix(symbol)
		} else {
			None
		}
	}

	fn get_unary_pos(&self) -> Option<OpUnary> {
		if let Some(symbol) = self.symbol() {
			OpUnary::get_posfix(symbol)
		} else {
			None
		}
	}

	fn get_binary(&self) -> Option<OpBinary> {
		if let Some(symbol) = self.symbol() {
			OpBinary::get(symbol)
		} else {
			None
		}
	}

	fn get_ternary(&self) -> Option<(OpTernary, &'static str)> {
		if let Some(symbol) = self.symbol() {
			OpTernary::get(symbol)
		} else {
			None
		}
	}
}

impl Atom {
	pub fn symbol(&self) -> Option<&str> {
		let Atom(value) = self;
		value.symbol()
	}

	fn resolve(&self, scope: &mut Scope, errors: &mut ErrorList) -> Option<Expr> {
		let Atom(value) = self;
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
					let expr = Expr::new(expr::Literal::String(value.clone()));
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
			errors.at(Some(span), format!("{value} is not a value"));
			None
		}
	}
}
