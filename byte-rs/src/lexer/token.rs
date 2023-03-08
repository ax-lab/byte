use super::Span;

#[derive(Clone)]
pub struct Lex {
	pub token: Token,
	pub span: Span,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Token {
	Break,
	Indent,
	Dedent,
	Identifier(String),
	Integer(u64),
	Literal(String),
	Symbol(&'static str),
}

impl Token {
	/// Returns the text for a symbolic token (either [`Token::Symbol`]
	/// or [`Token::Identifier`]).
	pub fn symbol(&self) -> Option<&str> {
		match self {
			Token::Identifier(s) => Some(s.as_str()),
			Token::Symbol(s) => Some(s),
			_ => None,
		}
	}

	/// Returns the closing symbol for an opening parenthesis token.
	pub fn closing(&self) -> Option<&'static str> {
		let right = match self {
			Token::Symbol("(") => ")",
			_ => return None,
		};
		Some(right)
	}
}

impl std::fmt::Display for Token {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Token::Identifier(id) => write!(f, "{id}"),
			Token::Integer(value) => write!(f, "{value}"),
			Token::Literal(value) => write!(f, "{value:?}"),
			Token::Symbol(symbol) => write!(f, "{symbol}"),
			token => write!(f, "{token:?}"),
		}
	}
}
