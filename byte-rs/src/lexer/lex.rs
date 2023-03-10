use super::{LexPosition, LexSource, Pos, Span, Token};

/// A lexeme from the source input.
#[derive(Copy, Clone)]
pub enum Lex<'a> {
	Some(LexPosition<'a>),
	None(&'a LexSource<'a>),
}

impl<'a> std::fmt::Display for Lex<'a> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Lex::Some(value) => {
				let token = value.token();
				match token {
					Token::Symbol(sym) => write!(f, "{sym}"),
					Token::Integer(value) => write!(f, "{value}"),
					Token::Literal(str) => {
						let Span { pos, end } = str.content_span();
						write!(
							f,
							"{:?}",
							value
								.source()
								.reader
								.source
								.read_text(pos.offset, end.offset)
						)
					}
					Token::Identifier => {
						write!(f, "{}", self.text())
					}
					_ => write!(f, "{token:?}"),
				}
			}
			Lex::None(..) => {
				write!(f, "end of input")
			}
		}
	}
}

impl<'a> Lex<'a> {
	pub fn from(source: &'a LexSource) -> Self {
		let state = LexPosition::from(source);
		Lex::Some(state)
	}

	pub fn is_some(&self) -> bool {
		match self {
			Lex::Some(_) => true,
			_ => false,
		}
	}

	pub fn next(&self) -> Self {
		match self {
			Lex::Some(state) => state.next(),
			Lex::None(source) => Lex::None(source),
		}
	}

	pub fn token(&self) -> Option<Token> {
		match self {
			Lex::Some(lex) => Some(lex.token()),
			Lex::None(..) => None,
		}
	}

	pub fn symbol(&self) -> Option<&str> {
		match self {
			Lex::Some(lex) => match lex.token() {
				Token::Symbol(str) => Some(str),
				Token::Identifier => Some(self.text()),
				_ => None,
			},
			_ => None,
		}
	}

	pub fn source(&self) -> &LexSource {
		match self {
			Lex::Some(lex) => lex.source(),
			Lex::None(src) => src,
		}
	}

	pub fn span(&self) -> Span {
		match self {
			Lex::Some(state) => state.span(),
			Lex::None(source) => {
				let pos = if let Some(last) = source.tokens.last() {
					last.1.end
				} else {
					Pos::default()
				};
				Span { pos, end: pos }
			}
		}
	}

	pub fn text(&self) -> &str {
		match self {
			Lex::Some(state) => {
				let span = state.span();
				state
					.source()
					.reader
					.source
					.read_text(span.pos.offset, span.end.offset)
			}
			Lex::None(_) => "",
		}
	}
}

// Read helpers.
impl<'a> Lex<'a> {
	/// Return the next token and true if the predicate matches the current
	/// token.
	pub fn next_if<F: Fn(Token) -> bool>(self, predicate: F) -> (Self, bool) {
		match self {
			Lex::None(_) => (self, false),
			Lex::Some(lex) => {
				let token = lex.token();
				if predicate(token) {
					(self.next(), true)
				} else {
					(self, false)
				}
			}
		}
	}

	/// Read the next token if it is the specific symbol.
	pub fn skip_symbol(self, symbol: &str) -> (Self, bool) {
		if self.symbol() == Some(symbol) {
			(self.next(), true)
		} else {
			(self, false)
		}
	}
}
