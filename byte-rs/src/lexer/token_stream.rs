use std::collections::VecDeque;

use super::{Input, LexerResult, Reader, Span, Token};

pub struct TokenStream<'a, T: Input> {
	input: &'a mut Reader<T>,
	next: VecDeque<(Token, Span)>,
	ident: VecDeque<usize>,
}

pub enum ReadToken<T> {
	MapTo(T),
	Unget(Token),
}

impl<'a, T: Input> TokenStream<'a, T> {
	pub fn new(input: &'a mut Reader<T>) -> TokenStream<'a, T> {
		TokenStream {
			input,
			next: Default::default(),
			ident: Default::default(),
		}
	}

	/// Span for the next token in the input for use in compiler messages.
	pub fn next_span(&mut self) -> Span {
		self.peek_next().1
	}

	/// Next token in the input. This is meant for use in compiler messages.
	pub fn next_token(&mut self) -> &Token {
		&self.peek_next().0
	}

	/// Read the next token in the input and passes it to the given parser
	/// function returning an [`Option`] result.
	///
	/// At the end of the input, returns [`None`] without calling the parser.
	pub fn read<V, F: FnOnce(&mut Self, Token, Span) -> Option<V>>(
		&mut self,
		parser: F,
	) -> Option<V> {
		let (token, span) = self.read_next();
		if let Token::None = token {
			None
		} else {
			parser(self, token, span)
		}
	}

	/// Try to read the next input passing it to the given parser function.
	///
	/// This is similar to `read` except that it can "unread" the token
	/// in case it does not match by returning [`ReadToken::Unget`].
	pub fn try_read<V, F: FnOnce(&mut Self, Token, Span) -> ReadToken<V>>(
		&mut self,
		consumer: F,
	) -> Option<V> {
		let (token, span) = self.read_next();
		match consumer(self, token, span) {
			ReadToken::MapTo(value) => Some(value),
			ReadToken::Unget(token) => {
				self.unget(token, span);
				None
			}
		}
	}

	/// Read the next token from the input and calls the parser function to
	/// map it to a value.
	///
	/// This will call the parser function even at the end of the input.
	pub fn map_next<V, F: FnOnce(Token, Span) -> V>(&mut self, parser: F) -> V {
		let (token, span) = self.read_next();
		parser(token, span)
	}

	/// Similar to `map_next` but does not consume the token if the mapping
	/// function returns [`None`].
	pub fn try_map_next<V, F: FnOnce(&Token, Span) -> Option<V>>(&mut self, map: F) -> Option<V> {
		let (token, span) = self.peek_next();
		if let Token::None = token {
			None
		} else if let Some(value) = map(token, *span) {
			self.shift();
			Some(value)
		} else {
			None
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
		let (token, _) = self.peek_next();
		if let Token::None = token {
			false
		} else if predicate(token) {
			self.shift();
			true
		} else {
			false
		}
	}

	/// Skip any number of tokens matching the given predicate.
	pub fn skip_while<F: Fn(&Token) -> bool>(&mut self, predicate: F) {
		while self.read_if(&predicate) {}
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
		let (token, span) = self.peek_next();
		if let Some(error) = predicate(token, *span) {
			Some(error)
		} else {
			self.shift();
			None
		}
	}

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

	/// Return a token to the stream to be read again.
	pub fn unget(&mut self, token: Token, span: Span) {
		let next = self.next_span();
		assert!(span.end.offset <= next.pos.offset);
		self.next.push_front((token, span));
	}

	/// This is meant for use when debugging the whole stream of tokens without
	/// parsing.
	pub fn dump_next(&mut self) -> (Token, Span, &str) {
		let (token, span) = self.read_next();
		let text = self.input.read_text(span);
		(token, span, text)
	}

	//------------------------------------------------------------------------//

	/// Return a child `TokenStream` that will iterate tokens from the
	/// current one, but will stop at a [`Token::Ident`].
	#[allow(unused)]
	pub fn indented(&mut self) -> TokenStream<T> {
		todo!()
	}

	/// Return a child `TokenStream` that will iterate tokens from the
	/// current one, but will stop at tokens for which `f` returns true.
	#[allow(unused)]
	pub fn until(&mut self, f: fn(Token) -> bool) -> TokenStream<T> {
		todo!()
	}

	/// Return a child `TokenStream` that will iterate tokens from the
	/// current one, but will stop at the given right parenthesis.
	///
	/// Note that this overrides an [`TokenStream::until`] limitation.
	#[allow(unused)]
	pub fn parenthesized(&mut self, right: Token) -> TokenStream<T> {
		todo!()
	}

	//------------------------------------------------------------------------//
	// Internal methods
	//------------------------------------------------------------------------//

	fn shift(&mut self) {
		self.next.pop_front().expect("shifting empty token");
	}

	fn peek_next(&mut self) -> &(Token, Span) {
		self.fill_next();
		self.next.front().unwrap()
	}

	fn read_next(&mut self) -> (Token, Span) {
		self.fill_next();
		self.next.pop_front().unwrap()
	}

	fn fill_next(&mut self) {
		while self.next.is_empty() {
			// check if we are at the start of the line so we can compute identation
			let start = self.input.pos();
			let new_line = start.column == 0;

			// read the next token
			let (result, span) = super::read_token(self.input);
			let token = match result {
				LexerResult::Token(token) => token,
				LexerResult::None => Token::None,
				LexerResult::Error(error) => panic!("{error} at {span}"),
			};

			let need_indent = (new_line && token != Token::LineBreak) || token == Token::None;
			let column = if token == Token::None {
				0
			} else {
				span.pos.column
			};
			if token != Token::Comment {
				self.next.push_back((token, span));
			}

			// check if we need indent or dedent tokens by comparing the first token level
			if need_indent {
				let ident = self.ident.back().copied().unwrap_or(0);
				if column > ident {
					self.ident.push_back(column);
					self.next.push_front((
						Token::Ident,
						Span {
							pos: start,
							end: span.pos,
						},
					));
				} else {
					let mut ident = ident;
					while column < ident {
						self.ident.pop_back();
						ident = self.ident.back().copied().unwrap_or(0);
						self.next.push_front((
							Token::Dedent,
							Span {
								pos: start,
								end: span.pos,
							},
						))
					}
				}
			}
		}
	}
}
