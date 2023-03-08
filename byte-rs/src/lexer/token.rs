use super::{LexSource, Span};

#[derive(Copy, Clone)]
pub struct LexState<'a> {
	index: usize,
	source: &'a LexSource,
}

impl<'a> LexState<'a> {
	pub fn span(&self) -> Span {
		self.source.tokens[self.index].1
	}

	pub fn token(&self) -> Token {
		self.source.tokens[self.index].0.clone()
	}
}

#[derive(Clone)]
pub enum Lex<'a> {
	Next(LexState<'a>),
	None,
}

impl<'a> Lex<'a> {
	pub fn new(source: &'a LexSource, index: usize) -> Self {
		if index < source.tokens.len() {
			let state = LexState { source, index };
			Lex::Next(state)
		} else {
			Lex::None
		}
	}

	#[allow(unused)]
	pub fn text(&self) -> &str {
		match self {
			Lex::Next(state) => {
				let span = state.span();
				state
					.source
					.reader
					.read_text(span.pos.offset, span.end.offset)
			}
			Lex::None => "",
		}
	}

	pub fn token(&self) -> Token {
		match self {
			Lex::Next(state) => state.token(),
			_ => unreachable!(),
		}
	}

	pub fn span(&self) -> Span {
		match self {
			Lex::Next(state) => state.span(),
			_ => unreachable!(),
		}
	}

	pub fn pair(&self) -> (Token, Span) {
		(self.token(), self.span())
	}
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Token {
	Break,
	Indent,
	Dedent,
	Identifier(String),
	Integer(u64),
	Literal(String),
	Symbol(&'static str),
}

impl Token {
	/// Returns the text for a symbolic token (either [`Token::Symbol`]
	/// or [`Token::Identifier`]).
	pub fn symbol(&self) -> Option<&str> {
		match self {
			Token::Identifier(s) => Some(s.as_str()),
			Token::Symbol(s) => Some(s),
			_ => None,
		}
	}

	/// Returns the closing symbol for an opening parenthesis token.
	pub fn closing(&self) -> Option<&'static str> {
		let right = match self {
			Token::Symbol("(") => ")",
			_ => return None,
		};
		Some(right)
	}
}

impl std::fmt::Display for Token {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Token::Identifier(id) => write!(f, "{id}"),
			Token::Integer(value) => write!(f, "{value}"),
			Token::Literal(value) => write!(f, "{value:?}"),
			Token::Symbol(symbol) => write!(f, "{symbol}"),
			token => write!(f, "{token:?}"),
		}
	}
}
