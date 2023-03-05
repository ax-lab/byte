#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Token {
	None,
	Comment,
	LineBreak,
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
