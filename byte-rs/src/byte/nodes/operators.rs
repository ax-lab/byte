use super::*;

/// Operator precedence for expression parsing.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum OpPrecedence {
	Lowest,
	Assign,
	BooleanNot,
	BooleanOr,
	BooleanAnd,
	Comparison,
	Additive,
	Multiplicative,
	Unary,
	Highest,
}
