use crate::core::input::Span;

use super::Token;

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Lex {
	pub token: Token,
	pub span: Span,
}

impl Lex {
	pub fn is_some(&self) -> bool {
		match self.token {
			Token::None => false,
			_ => true,
		}
	}

	pub fn as_none(&self) -> Lex {
		Lex {
			token: Token::None,
			span: Span {
				sta: self.span.sta,
				end: self.span.sta,
			},
		}
	}

	pub fn symbol(&self) -> Option<&str> {
		match self.token {
			Token::Symbol(str) => Some(str),
			Token::Identifier => Some(self.text()),
			_ => None,
		}
	}

	pub fn text(&self) -> &'static str {
		self.span.text()
	}
}

impl std::fmt::Debug for Lex {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Lex")
			.field("token", &self.token)
			.field("span", &self.span)
			.finish()
	}
}

impl std::fmt::Display for Lex {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self.token {
			Token::None => {
				write!(f, "end of input")
			}
			Token::Invalid => {
				write!(f, "invalid token")
			}
			token => match token {
				Token::Symbol(sym) => write!(f, "{sym}"),
				Token::Integer(value) => write!(f, "{value}"),
				Token::Literal(content) => {
					write!(f, "{:?}", content)
				}
				Token::Identifier => {
					write!(f, "{}", self.text())
				}
				_ => write!(f, "{token:?}"),
			},
		}
	}
}
