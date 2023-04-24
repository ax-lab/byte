use std::io::Write;

use crate::core::repr::*;
use crate::core::traits::*;
use crate::core::*;

use input::*;

/// Trait for custom token types returned as [`Token::Other`].
pub trait IsToken: Sized {
	type Value: IsValue + HasEq;

	fn name() -> &'static str;

	fn token(data: Self::Value) -> Token {
		let data = TokenValue::new::<Self>(data);
		Token::Other(data)
	}
}

/// Enumeration of possible tokens for the language.
///
/// ## Custom tokens
///
/// Additional tokens can be defined by implementing the [`IsToken`] trait
/// and a custom [`crate::lexer::Matcher`].
///
/// Those custom tokens are returned as [`Token::Other`] and can be tested
/// and retrieved using the [`Token::is`] and [`Token::get`] methods.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Token {
	None,
	Invalid,
	Break,
	Indent,
	Dedent,
	Identifier,
	Symbol(&'static str),
	Other(TokenValue),
}

impl Token {
	pub fn is<T: IsToken>(&self) -> bool {
		match self {
			Token::Other(value) => value.get::<T>().is_some(),
			_ => false,
		}
	}

	pub fn get<T: IsToken>(&self) -> Option<&T::Value> {
		match self {
			Token::Other(data) => data.get::<T>(),
			_ => None,
		}
	}
}

/// Wraps a single instance of a [`Token`] and its position in the stream.
///
/// This provides token's [`Span`] position, which provides access to the raw
/// text for the token. This is useful for error messages and is sometimes
/// necessary to parse the value of the token.
#[derive(Clone, PartialEq)]
pub struct TokenAt(pub Span, pub Token);

impl TokenAt {
	pub fn span(&self) -> Span {
		self.0.clone()
	}

	pub fn token(&self) -> Token {
		self.1.clone()
	}

	pub fn column(&self) -> usize {
		self.span().sta.col()
	}

	pub fn indent(&self) -> usize {
		self.span().sta.indent()
	}

	pub fn first_of_line(&self) -> bool {
		self.column() == self.indent()
	}

	pub fn symbol(&self) -> Option<&str> {
		let str = match &self.1 {
			Token::Symbol(symbol) => *symbol,
			Token::Identifier => self.0.text(),
			_ => return None,
		};
		Some(str)
	}

	pub fn is_some(&self) -> bool {
		self.1 != Token::None
	}

	pub fn is_none(&self) -> bool {
		self.1 == Token::None
	}

	pub fn text(&self) -> &str {
		self.0.text()
	}

	pub fn as_none(&self) -> TokenAt {
		TokenAt(self.span(), Token::None)
	}
}

impl HasRepr for TokenAt {
	fn output_repr(&self, output: &mut Repr) -> std::io::Result<()> {
		if output.is_debug() {
			write!(output, "<{:?}", self.1)?;
			if output.is_full() {
				write!(output, " ")?;
				self.0.output_repr(output)?;
			}
			write!(output, ">")?;
		} else {
			match &self.1 {
				Token::None => {
					write!(output, "end of input")?;
				}
				Token::Invalid => {
					write!(output, "invalid token")?;
				}
				Token::Break => {
					write!(output, "line break")?;
				}
				Token::Indent => {
					write!(output, "indent")?;
				}
				Token::Dedent => {
					write!(output, "dedent")?;
				}
				Token::Symbol(sym) => write!(output, "`{sym}`")?,
				Token::Identifier => {
					write!(output, "`{}`", self.span().text())?;
				}
				Token::Other(value) => value.output_repr(output)?,
			}
		}
		Ok(())
	}
}

fmt_from_repr!(TokenAt);

/// Holds the value of a custom [`Token::Other`].
#[derive(Clone, PartialEq, Eq)]
pub struct TokenValue {
	name: &'static str,
	data: Value,
}

impl TokenValue {
	pub fn new<T: IsToken>(data: T::Value) -> Self {
		Self {
			name: T::name(),
			data: Value::from(data),
		}
	}

	pub fn get<T: IsToken>(&self) -> Option<&T::Value> {
		self.data.get()
	}
}

impl HasRepr for TokenValue {
	fn output_repr(&self, output: &mut Repr) -> std::io::Result<()> {
		if output.is_debug() {
			write!(output, "<{} ", self.name)?;
			self.data.output_repr(output)?;
			write!(output, ">")?;
		} else {
			self.data.output_repr(output)?;
		}
		Ok(())
	}
}

fmt_from_repr!(TokenValue);
