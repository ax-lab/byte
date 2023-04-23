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

impl std::fmt::Display for TokenAt {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match &self.1 {
			Token::None => {
				write!(f, "end of input")
			}
			Token::Invalid => {
				write!(f, "invalid token")
			}
			Token::Break => {
				write!(f, "line break")
			}
			Token::Indent => {
				write!(f, "indent")
			}
			Token::Dedent => {
				write!(f, "dedent")
			}
			Token::Symbol(sym) => write!(f, "`{sym}`"),
			Token::Identifier => {
				write!(f, "`{}`", self.span().text())
			}
			Token::Other(value) => write!(f, "{value}"),
		}
	}
}

impl std::fmt::Debug for TokenAt {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "<{:?} {:?}>", self.1, self.0)
	}
}

/// Holds the value of a custom [`Token::Other`].
#[derive(Clone, PartialEq)]
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

impl std::fmt::Display for TokenValue {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		if self.data.get::<()>().is_none() {
			write!(f, "{} `{}`", self.name, self.data)
		} else {
			write!(f, "{}", self.name)
		}
	}
}

impl Eq for TokenValue {}

impl std::fmt::Debug for TokenValue {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}{{ ", self.name)?;
		self.data.fmt(f)?;
		write!(f, " }}")
	}
}
