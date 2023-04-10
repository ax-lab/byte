use crate::core::error::*;
use crate::lexer::*;

/// Errors generated by the lexer.
#[derive(Debug)]
pub enum ParserError {
	ExpectedEnd(TokenAt),
}

impl IsError for ParserError {
	fn output(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			ParserError::ExpectedEnd(got) => {
				write!(f, "expected statement end, got `{got}`")
			}
		}
	}
}
