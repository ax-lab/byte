use crate::lexer::{Lex, LexerError};
use crate::Span;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug)]
pub enum Error {
	At(String, Box<Error>),
	Lexer(LexerError, Span),
	Dedent(Span),
	ClosingSymbol(&'static str, Span),
	ClosingDedent(&'static str, Span),
	Expected(&'static str, Lex),
	ExpectedEnd(Lex),
	ExpectedExpression(Lex),
	ExpectedSymbol(&'static str, Span),
	ExpectedIndent(Span),
	InvalidToken(Span),
}

impl Error {
	pub fn span(&self) -> Span {
		match self {
			Error::At(_, err) => err.span(),
			Error::Lexer(_, span) => *span,
			Error::Dedent(span) => *span,
			Error::ClosingSymbol(_, span) => *span,
			Error::ClosingDedent(_, span) => *span,
			Error::Expected(_, lex) => lex.span,
			Error::ExpectedEnd(lex) => lex.span,
			Error::ExpectedExpression(lex) => lex.span,
			Error::ExpectedSymbol(_, span) => *span,
			Error::ExpectedIndent(span) => *span,
			Error::InvalidToken(span) => *span,
		}
	}

	pub fn at<T: Into<String>>(self, context: T) -> Error {
		Error::At(context.into(), self.into())
	}
}

impl<T> Into<Result<T>> for Error {
	fn into(self) -> Result<T> {
		Result::Err(self)
	}
}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Error::At(context, error) => write!(f, "{context}: {error}"),
			Error::Lexer(error, span) => error.output(f, *span),
			Error::Dedent(..) => write!(f, "unexpected dedent"),
			Error::ClosingSymbol(sym, ..) => write!(f, "unexpected closing `{sym}`"),
			Error::ClosingDedent(sym, ..) => write!(f, "unexpected dedent before closing `{sym}`"),
			Error::Expected(what, sym) => write!(f, "expected {what}, got `{sym}`"),
			Error::ExpectedEnd(sym) => write!(f, "expected end, got `{sym}`"),
			Error::ExpectedExpression(sym) => write!(f, "expression expected, got `{sym}`"),
			Error::ExpectedSymbol(sym, ..) => write!(f, "expected `{sym}`"),
			Error::ExpectedIndent(..) => write!(f, "expected indented line"),
			Error::InvalidToken(..) => write!(f, "invalid token, parsing failed"),
		}
	}
}
