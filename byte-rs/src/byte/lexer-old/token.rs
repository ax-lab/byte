use std::io::Write;

use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Token {
	Invalid,
	Break,
	Symbol(&'static str),
	Word(Span),
}

has_traits!(Token: IsNode, WithEquality);

impl IsNode for Token {}

impl HasRepr for Token {
	fn output_repr(&self, output: &mut Repr) -> std::io::Result<()> {
		let debug = output.is_debug();
		if debug {
			write!(output, "<")?;
		}
		match self {
			Token::Invalid => {
				write!(
					output,
					"{}",
					if debug { "Invalid" } else { "invalid token" }
				)?;
			}
			Token::Break => {
				write!(output, "{}", if debug { "Break" } else { "line break" })?;
			}
			Token::Symbol(str) => {
				if debug {
					write!(output, "Sym({str:?})")?;
				} else {
					write!(output, "{str}")?;
				}
			}
			Token::Word(str) => {
				if debug {
					write!(output, "Word({str})")?;
				} else {
					write!(output, "{str}")?;
				}
			}
		}
		if debug {
			write!(output, ">")?;
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

	pub fn is_break(&self) -> bool {
		self.is_token(|x| matches!(x, Token::Break))
	}

	pub fn symbol(&self) -> Option<&str> {
		if let Some(token) = self.get_token() {
			match token {
				Token::Symbol(str) => Some(*str),
				Token::Word(str) => Some(str.as_str()),
				_ => None,
			}
		} else {
			None
		}
	}

	pub fn is_symbol(&self, symbol: &str) -> bool {
		self.symbol() == Some(symbol)
	}
}
