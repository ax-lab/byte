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
	pub fn token(&self) -> Option<&Token> {
		if let Bit::Token(token) = self.bit() {
			Some(token)
		} else {
			None
		}
	}
}
