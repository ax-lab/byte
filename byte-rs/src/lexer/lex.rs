use super::{lex_string::LexString, Input, Range, Token};

#[derive(Copy, Clone)]
pub struct Lex<'a> {
	pub token: Token,
	pub range: Range<'a>,
}

impl<'a> Lex<'a> {
	pub fn is_some(&self) -> bool {
		match self.token {
			Token::None => false,
			_ => true,
		}
	}

	pub fn symbol(&self) -> Option<&str> {
		match self.token {
			Token::Symbol(str) => Some(str),
			Token::Identifier => Some(self.text()),
			_ => None,
		}
	}

	pub fn source(&self) -> &'a dyn Input {
		self.range.pos.source
	}

	pub fn text(&self) -> &str {
		self.range.text()
	}
}

impl<'a> std::fmt::Debug for Lex<'a> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Lexeme")
			.field("token", &self.token)
			.field("range", &self.range)
			.finish()
	}
}

impl<'a> std::fmt::Display for Lex<'a> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self.token {
			Token::None => {
				write!(f, "end of input")
			}
			token => match token {
				Token::Symbol(sym) => write!(f, "{sym}"),
				Token::Integer(value) => write!(f, "{value}"),
				Token::Literal(LexString { pos, end }) => {
					write!(f, "{:?}", self.source().read_text(pos, end))
				}
				Token::Identifier => {
					write!(f, "{}", self.text())
				}
				_ => write!(f, "{token:?}"),
			},
		}
	}
}
