pub use super::*;

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
