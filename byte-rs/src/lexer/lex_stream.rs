use std::collections::VecDeque;

use super::{Input, LexerResult, Reader, Span, Token};

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

	fn next(mut self) -> Option<Self> {
		if self.index < self.source.tokens.len() - 1 {
			self.index += 1;
			Some(self)
		} else {
			None
		}
	}
}

/// A lexeme from the source input.
#[derive(Copy, Clone)]
pub enum Lex<'a> {
	Some(LexPosition<'a>),
	None,
}

impl<'a> Lex<'a> {
	fn new(source: &'a LexSource, index: usize) -> Self {
		if index < source.tokens.len() {
			let state = LexPosition { source, index };
			Lex::Some(state)
		} else {
			Lex::None
		}
	}

	pub fn next(&self) -> Self {
		match self {
			Lex::Some(state) => {
				if let Some(next) = state.next() {
					Lex::Some(next)
				} else {
					Lex::None
				}
			}
			Lex::None => Lex::None,
		}
	}

	#[allow(unused)]
	pub fn text(&self) -> &str {
		match self {
			Lex::Some(state) => {
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
			Lex::Some(state) => state.token(),
			_ => unreachable!(),
		}
	}

	pub fn span(&self) -> Span {
		match self {
			Lex::Some(state) => state.span(),
			_ => unreachable!(),
		}
	}

	pub fn pair(&self) -> (Token, Span) {
		(self.token(), self.span())
	}
}

#[derive(Copy, Clone)]
pub struct LexStream<'a> {
	source: &'a LexSource,
	next: Lex<'a>,
}

pub enum Parse<T> {
	As(T),
	None,
}

impl<'a> LexStream<'a> {
	pub fn new(source: &'a LexSource) -> LexStream<'a> {
		LexStream {
			source,
			next: Lex::new(source, 0),
		}
	}

	pub fn read_text(&self, pos: usize, end: usize) -> &str {
		self.source.reader.read_text(pos, end)
	}

	/// Read the next (token, span) pair.
	pub fn read(&mut self) -> Lex {
		let output = self.next;
		self.shift();
		output
	}

	/// Next token in the input. Should be used only for compiler messages.
	pub fn next(&mut self) -> Lex {
		self.next
	}

	pub fn at_end(&self) -> bool {
		matches!(self.next, Lex::None)
	}

	/// Read the next token in the input and passes it to the given parser
	/// function returning an [`Option`] result.
	///
	/// At the end of the input, returns [`None`] without calling the parser.
	pub fn parse<V, F: FnOnce(&mut Self, Token, Span) -> Option<V>>(
		&mut self,
		parser: F,
	) -> Option<V> {
		match self.read() {
			Lex::None => None,
			Lex::Some(lex) => {
				let token = lex.token();
				let span = lex.span();
				let out = parser(self, token, span);
				out
			}
		}
	}

	/// Try to read the next input passing it to the given parser function.
	///
	/// This is similar to `read` except that it can "unread" the token
	/// in case it does not match by returning [`ReadToken::Unget`].
	pub fn try_read<V, F: FnOnce(&mut Self, Token, Span) -> Parse<V>>(
		&mut self,
		consumer: F,
	) -> Option<V> {
		let saved = self.next;
		match self.read() {
			Lex::None => None,
			Lex::Some(lex) => {
				let token = lex.token();
				let span = lex.span();
				let out = match consumer(self, token, span) {
					Parse::As(value) => Some(value),
					Parse::None => {
						self.next = saved;
						None
					}
				};
				out
			}
		}
	}

	/// Read the next token from the input and calls the parser function to
	/// map it to a value.
	///
	/// This will call the parser function even at the end of the input.
	pub fn map_next<V, F: FnOnce(Token, Span) -> V>(&mut self, parser: F) -> V {
		match self.read() {
			Lex::None => panic!("no next token in map_next"),
			Lex::Some(lex) => parser(lex.token(), lex.span()),
		}
	}

	/// Similar to `map_next` but does not consume the token if the mapping
	/// function returns [`None`].
	pub fn try_map_next<V, F: FnOnce(&Token, Span) -> Option<V>>(&mut self, map: F) -> Option<V> {
		match self.next() {
			Lex::None => None,
			Lex::Some(lex) => {
				let value = {
					let token = lex.token();
					let span = lex.span();
					if let Some(value) = map(&token, span) {
						Some(value)
					} else {
						None
					}
				};
				if value.is_some() {
					self.shift();
				}
				value
			}
		}
	}

	/// Consume the next token if it is a symbol and it has a mapping returned
	/// by the given mapper function.
	///
	/// Returns the symbol mapping or None if not matched.
	pub fn map_symbol<V, F: FnOnce(&str) -> Option<V>>(&mut self, mapper: F) -> Option<V> {
		self.try_map_next(|token, _| {
			if let Some(symbol) = token.symbol() {
				mapper(symbol)
			} else {
				None
			}
		})
	}

	/// Read the next token if it matches the given predicate.
	///
	/// Returns `true` if the token was read or `false` if the predicate
	/// was not true or at the end of input.
	pub fn read_if<F: Fn(&Token) -> bool>(&mut self, predicate: F) -> bool {
		match self.next() {
			Lex::None => false,
			Lex::Some(lex) => {
				let token = lex.token();
				if predicate(&token) {
					self.shift();
					true
				} else {
					false
				}
			}
		}
	}

	/// Read the next token if it is the specific symbol.
	pub fn read_symbol(&mut self, symbol: &str) -> bool {
		self.read_if(|token| {
			if let Some(next) = token.symbol() {
				next == symbol
			} else {
				false
			}
		})
	}

	/// Validate the next token using the given predicate. If the predicate
	/// returns an error, returns that error without consuming the token.
	///
	/// On success, returns None.
	pub fn expect<E, F: FnOnce(&Token, Span) -> Option<E>>(&mut self, predicate: F) -> Option<E> {
		let error = {
			let next = self.next();
			let token = next.token();
			let span = next.span();
			predicate(&token, span)
		};

		if let Some(error) = error {
			Some(error)
		} else {
			self.shift();
			None
		}
	}

	//------------------------------------------------------------------------//
	// Parsing helpers
	//------------------------------------------------------------------------//

	/// Helper for expecting a specific symbol token.
	pub fn expect_symbol<E, P: FnOnce(Span) -> E>(&mut self, symbol: &str, error: P) -> Option<E> {
		self.expect(|token, span| {
			if let Some(actual) = token.symbol() {
				if actual == symbol {
					return None;
				}
			}
			Some(error(span))
		})
	}

	pub fn expect_dedent(&mut self) -> Option<(String, Span)> {
		self.expect(|token, span| {
			if let Token::Dedent = token {
				None
			} else {
				Some((format!("expected dedent, got `{token}`"), span))
			}
		})
	}

	//------------------------------------------------------------------------//
	// Internal methods
	//------------------------------------------------------------------------//

	fn shift(&mut self) {
		self.next = self.next.next();
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
			if Some(closing) == token.symbol() {
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
