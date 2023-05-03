use std::io::Write;

use super::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Token {
	None,
	Invalid,
	Break,
	Identifier(Span),
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
				Token::None => {
					write!(output, "end of input")?;
				}
				Token::Invalid => {
					write!(output, "invalid token")?;
				}
				Token::Break => {
					write!(output, "line break")?;
				}
				Token::Symbol(sym) => write!(output, "`{sym}`")?,
				Token::Identifier(span) => {
					write!(output, "`{}`", span.text())?;
				}
			}
		}
		Ok(())
	}
}
