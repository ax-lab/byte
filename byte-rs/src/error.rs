use crate::lexer::{Lex, LexerError, Span};

pub type Result<'a, T> = std::result::Result<T, Error<'a>>;

#[derive(Clone, Debug)]
pub enum Error<'a> {
	At(String, Box<Error<'a>>),
	Lexer(LexerError, Span<'a>),
	Dedent(Span<'a>),
	ClosingSymbol(&'static str, Span<'a>),
	ClosingDedent(&'static str, Span<'a>),
	ExpectedEnd(Lex<'a>),
	ExpectedExpression(Span<'a>),
	ExpectedSymbol(&'static str, Span<'a>),
	ExpectedIndent(Span<'a>),
	InvalidToken(Span<'a>),
}

impl<'a> Error<'a> {
	pub fn span(&self) -> Span<'a> {
		match self {
			Error::At(_, err) => err.span(),
			Error::Lexer(_, span) => *span,
			Error::Dedent(span) => *span,
			Error::ClosingSymbol(_, span) => *span,
			Error::ClosingDedent(_, span) => *span,
			Error::ExpectedEnd(lex) => lex.span,
			Error::ExpectedExpression(span) => *span,
			Error::ExpectedSymbol(_, span) => *span,
			Error::ExpectedIndent(span) => *span,
			Error::InvalidToken(span) => *span,
		}
	}

	pub fn at<T: Into<String>>(self, context: T) -> Error<'a> {
		Error::At(context.into(), self.into())
	}
}

impl<'a, T> Into<Result<'a, T>> for Error<'a> {
	fn into(self) -> Result<'a, T> {
		Result::Err(self)
	}
}

impl<'a> std::fmt::Display for Error<'a> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Error::At(context, error) => write!(f, "{context}: {error}"),
			Error::Lexer(error, span) => error.output(f, *span),
			Error::Dedent(..) => write!(f, "unexpected dedent"),
			Error::ClosingSymbol(sym, ..) => write!(f, "unexpected closing `{sym}`"),
			Error::ClosingDedent(sym, ..) => write!(f, "unexpected dedent before closing `{sym}`"),
			Error::ExpectedEnd(sym) => write!(f, "expected end, got `{sym}`"),
			Error::ExpectedExpression(..) => write!(f, "expression expected"),
			Error::ExpectedSymbol(sym, ..) => write!(f, "expected `{sym}`"),
			Error::ExpectedIndent(..) => write!(f, "expected indented line"),
			Error::InvalidToken(..) => write!(f, "invalid token, parsing failed"),
		}
	}
}
