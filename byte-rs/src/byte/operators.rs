use super::*;

pub mod bracket;
pub mod indent;
pub mod line;
pub mod op_binary;
pub mod op_ternary;
pub mod op_unary;

pub use bracket::*;
pub use indent::*;
pub use line::*;
pub use op_binary::*;
pub use op_ternary::*;
pub use op_unary::*;

#[derive(Clone, Eq, PartialEq, Hash)]
pub enum Operator {
	Tokenize,
	SplitLines,
}

impl Operator {
	pub fn precedence(&self) -> Precedence {
		match self {
			Operator::Tokenize => Precedence::Lexer,
			Operator::SplitLines => Precedence::LineSplit,
		}
	}
}

/// Global evaluation precedence for language nodes.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Precedence {
	First,
	Lexer,
	LineSplit,
	Last,
}
