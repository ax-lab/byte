use std::io::Write;

use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Token {
	Invalid,
	Break,
	Symbol(&'static str),
}

has_traits!(Token: IsNode, WithEquality);

impl IsNode for Token {}

impl HasRepr for Token {
	fn output_repr(&self, output: &mut Repr<'_>) -> std::io::Result<()> {
		if output.is_debug() {
			write!(output, "<{:?}>", self)?;
		} else {
			match self {
				Token::Invalid => {
					write!(output, "invalid token")?;
				}
				Token::Break => {
					write!(output, "line break")?;
				}
				Token::Symbol(sym) => write!(output, "`{sym}`")?,
			}
		}
		Ok(())
	}
}

impl Node {
	pub fn get_token(&self) -> Option<&Token> {
		self.get::<Token>()
	}

	pub fn is_token<T: FnOnce(&Token) -> bool>(&self, pred: T) -> bool {
		self.get_token().map(|x| pred(x)).unwrap_or_default()
	}

	pub fn symbol(&self) -> Option<&str> {
		if let Some(token) = self.get_token() {
			match token {
				Token::Symbol(symbol) => Some(*symbol),
				_ => None,
			}
		} else if let Some(id) = self.get_identifier() {
			Some(id.value())
		} else {
			None
		}
	}

	pub fn is_symbol(&self, symbol: &str) -> bool {
		self.symbol() == Some(symbol)
	}
}
