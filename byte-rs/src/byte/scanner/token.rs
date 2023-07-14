pub use super::*;

/// Low level tokens generated directly by the [`Matcher`].
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum Token {
	Break(usize),
	Comment,
	Word(Symbol),
	Symbol(Symbol),
	Literal(StringValue),
	Integer(u128),
	Float(StringValue),
}

impl Node {
	pub fn token(&self) -> Option<Token> {
		if let NodeValue::Token(token) = self.val() {
			Some(token)
		} else {
			None
		}
	}
}

impl Display for Token {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		match self {
			Token::Break(..) => write!(f, "line break"),
			Token::Comment => write!(f, "comment"),
			Token::Word(s) => write!(f, "{s}"),
			Token::Symbol(s) => write!(f, "{s}"),
			Token::Literal(v) => write!(f, "{v:?}"),
			Token::Integer(v) => write!(f, "{v}"),
			Token::Float(v) => write!(f, "{v}"),
		}
	}
}
