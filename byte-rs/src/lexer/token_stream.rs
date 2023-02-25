use std::{cell::RefCell, rc::Rc};

use super::{Span, Token, TokenSource};

pub struct TokenStream<T: TokenSource> {
	input: Rc<RefCell<T>>,
}

pub enum ReadToken<T> {
	MapTo(T),
	Unget(Token),
}

impl<T: TokenSource> TokenStream<T> {
	pub fn new(input: T) -> TokenStream<T> {
		TokenStream {
			input: Rc::new(input.into()),
		}
	}

	/// Span for the next token in the input for use in compiler messages.
	pub fn next_span(&mut self) -> Span {
		let next = self.input.borrow();
		let next = next.peek();
		next.1
	}

	/// Next token in the input. This is meant for use in compiler messages.
	pub fn next_token(&mut self) -> Token {
		let next = self.input.borrow();
		let next = next.peek();
		next.0.clone()
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
		if let Token::None = token {
			None
		} else {
			match consumer(self, token, span) {
				ReadToken::MapTo(value) => Some(value),
				ReadToken::Unget(token) => {
					self.unget(token, span);
					None
				}
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
		let value = {
			let next = self.input.borrow();
			let next = next.peek();
			let token = &next.0;
			let span = next.1;
			if let Token::None = token {
				None
			} else if let Some(value) = map(token, span) {
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
		let res = {
			let next = self.input.borrow();
			let next = next.peek();
			let token = &next.0;
			if let Token::None = token {
				false
			} else if predicate(token) {
				true
			} else {
				false
			}
		};
		if res {
			self.shift();
		}
		res
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
		let error = {
			let next = self.input.borrow();
			let next = next.peek();
			let token = &next.0;
			let span = next.1;
			predicate(token, span)
		};

		if let Some(error) = error {
			Some(error)
		} else {
			self.shift();
			None
		}
	}

	/// Return a token to the stream to be read again.
	pub fn unget(&mut self, token: Token, span: Span) {
		self.input.borrow_mut().unget(token, span);
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
		self.input.borrow_mut().read();
	}

	fn read_next(&mut self) -> (Token, Span) {
		self.input.borrow_mut().read()
	}
}
