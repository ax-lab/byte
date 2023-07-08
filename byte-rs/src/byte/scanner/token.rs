pub use super::*;

#[derive(Clone, Debug)]
pub struct Lexeme(Token, Id);

/// Low level tokens generated directly by the [`Matcher`].
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum Token {
	Break,
	Indent(usize),
	Comment,
	Word(Symbol),
	Symbol(Symbol),
	Literal(StringValue),
	Integer(u128),
}

impl Lexeme {
	pub fn id(&self) -> &Id {
		&self.1
	}

	pub fn token(&self) -> &Token {
		&self.0
	}
}
