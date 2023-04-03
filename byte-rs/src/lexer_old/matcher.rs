use crate::core::input::*;

use super::{LexerError, Token};

mod comment;
mod identifier;
mod line_break;
mod literal;
mod number;
mod space;
mod symbol;

pub use comment::MatchComment;
pub use identifier::MatchIdentifier;
pub use line_break::MatchLineBreak;
pub use literal::MatchLiteral;
pub use number::MatchNumber;
pub use space::MatchSpace;
pub use symbol::{MatchSymbol, SymbolTable};

pub trait Matcher {
	/// Tries to read the next recognized token from the input.
	///
	/// Returns [`LexerResult::None`] if the next token is not recognized or
	/// at the end of input.
	///
	/// The input will advance to the end of the recognized token iff the
	/// token is recognized.
	fn try_match(&self, next: char, input: &mut Cursor) -> MatcherResult;

	fn clone_box(&self) -> Box<dyn Matcher>;
}

#[derive(Debug)]
pub enum MatcherResult {
	None,
	Skip,
	Comment,
	Token(Token),
	Error(LexerError),
}
