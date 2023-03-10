use std::collections::VecDeque;

use super::{Input, LexerResult, Pos, Reader, Span, Token};

/// Wraps the input reader and its list of tokens.
pub struct LexSource {
	pub reader: Reader,
	pub tokens: Vec<(Token, Span)>,
}

impl LexSource {
	pub fn new<T: Input + 'static>(input: T) -> LexSource {
		let reader = Reader::from(input);
		let tokens = read_all(reader.clone());
		LexSource { reader, tokens }
	}

	pub fn first(&self) -> Lex {
		Lex::new(self, 0)
	}

	pub fn read_text(&self, span: Span) -> &str {
		self.reader.read_text(span.pos.offset, span.end.offset)
	}
}

/// Represents a valid token position in the source.
#[derive(Copy, Clone)]
pub struct LexPosition<'a> {
	index: usize,
	source: &'a LexSource,
}

impl<'a> LexPosition<'a> {
	pub fn span(&self) -> Span {
		self.source.tokens[self.index].1
	}

	pub fn token(&self) -> Token {
		self.source.tokens[self.index].0.clone()
	}

	pub fn pair(&self) -> (Token, Span) {
		self.source.tokens[self.index].clone()
	}

	pub fn triple(&self) -> (Token, Span, &str) {
		let pair = self.pair();
		(pair.0, pair.1, self.text())
	}

	pub fn next(mut self) -> Lex<'a> {
		if self.index < self.source.tokens.len() - 1 {
			self.index += 1;
			Lex::Some(self)
		} else {
			Lex::None(self.source)
		}
	}

	pub fn text(&self) -> &str {
		let span = self.span();
		self.source
			.reader
			.read_text(span.pos.offset, span.end.offset)
	}
}

/// A lexeme from the source input.
#[derive(Copy, Clone)]
pub enum Lex<'a> {
	Some(LexPosition<'a>),
	None(&'a LexSource),
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
							value.source.reader.read_text(pos.offset, end.offset)
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
	fn new(source: &'a LexSource, index: usize) -> Self {
		if index < source.tokens.len() {
			let state = LexPosition { source, index };
			Lex::Some(state)
		} else {
			Lex::None(source)
		}
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
			Lex::Some(lex) => lex.source,
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
					.source
					.reader
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

fn read_all(mut input: Reader) -> Vec<(Token, Span)> {
	let mut tokens = Vec::new();
	let mut indent = VecDeque::new();
	let mut parens = VecDeque::new();

	let mut is_last = false;
	let mut is_empty = true;
	while !is_last {
		// check if we are at the start of the line so we can compute indentation
		let start = input.pos();
		let new_line = start.column == 0;
		is_empty = is_empty || new_line;

		// read the next token
		let (result, span) = super::read_token(&mut input);
		let (token, end, skip) = match result {
			LexerResult::Token(token) => (token, false, false),
			LexerResult::None => (Token::Break, true, false),
			LexerResult::Skip => (Token::Symbol(""), false, true),
			LexerResult::Error(error) => panic!("{error} at {span}"),
		};

		let (closing, closing_level) = if let Some(&(closing, level)) = parens.back() {
			let symbol = match token {
				Token::Symbol(symbol) => Some(symbol),
				Token::Identifier => Some(input.read_text(span.pos.offset, span.end.offset)),
				_ => None,
			};
			if Some(closing) == symbol {
				parens.pop_back();
				(true, level)
			} else {
				(false, 0)
			}
		} else {
			(false, 0)
		};

		let need_indent = (new_line && token != Token::Break) || end || closing;
		let column = if end {
			0
		} else if closing {
			closing_level
		} else {
			span.pos.column
		};

		// check if we need indent or dedent tokens by comparing the first token level
		if need_indent {
			let level = indent.back().copied().unwrap_or(0);
			if column > level {
				indent.push_back(column);
				tokens.push((
					Token::Indent,
					Span {
						pos: start,
						end: span.pos,
					},
				));
			} else {
				let mut level = level;
				while column < level {
					indent.pop_back();
					level = indent.back().copied().unwrap_or(0);
					tokens.push((
						Token::Dedent,
						Span {
							pos: start,
							end: span.pos,
						},
					));
				}
			}
		}

		if let Some(closing) = token.closing() {
			let level = indent.back().copied().unwrap_or(0);
			parens.push_back((closing, level));
		}

		is_last = end;
		let skip = skip
			|| match token {
				Token::Break => is_empty || end,
				_ => false,
			};

		if !skip {
			is_empty = false;
			tokens.push((token, span));
		}
	}
	tokens
}
