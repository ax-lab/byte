use std::io::Write;

use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Token {
	EndOfInput,
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
				Token::EndOfInput => {
					write!(output, "end of input")?;
				}
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
	pub fn get_token(&self) -> Option<Token> {
		self.get::<Token>().cloned()
	}

	pub fn is_token<T: FnOnce(Token) -> bool>(&self, pred: T) -> bool {
		self.get_token().map(|x| pred(x)).unwrap_or_default()
	}

	pub fn is_end(&self) -> bool {
		self.is_token(|x| x == Token::EndOfInput)
	}
}
