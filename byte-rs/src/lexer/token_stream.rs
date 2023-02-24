use std::collections::VecDeque;

use super::{Input, LexerResult, Reader, Span, Token};

pub struct TokenStream<'a, T: Input> {
	input: &'a mut Reader<T>,
	next: VecDeque<(Token, Span)>,
	ident: VecDeque<usize>,
}

pub enum WithToken<T> {
	Read(T),
	None(Token),
}

impl<'a, T: Input> TokenStream<'a, T> {
	pub fn new(input: &'a mut Reader<T>) -> TokenStream<'a, T> {
		TokenStream {
			input,
			next: Default::default(),
			ident: Default::default(),
		}
	}

	/// Consume the next token if it is a symbol and it has a mapping returned
	/// by the given mapper function.
	///
	/// Returns the symbol mapping or None if not matched.
	pub fn read_symbol<V, F: FnOnce(&str) -> Option<V>>(&mut self, mapper: F) -> Option<V> {
		let (token, _) = self.peek_next();
		if let Some(symbol) = token.symbol() {
			if let Some(value) = mapper(symbol) {
				self.next.pop_front();
				Some(value)
			} else {
				None
			}
		} else {
			None
		}
	}

	pub fn read_symbol_str(&mut self, symbol: &str) -> bool {
		self.read_symbol(|next| if next == symbol { Some(true) } else { None })
			.unwrap_or_default()
	}

	/// Consume the next token if it is the exact given symbol. Otherwise, call
	/// the error predicate and return it's error.
	pub fn expect_symbol<E, P: FnOnce(Span) -> E>(&mut self, symbol: &str, error: P) -> Option<E> {
		match self.read_symbol(|actual| if actual == symbol { Some(()) } else { None }) {
			Some(_) => None,
			None => Some(error(self.next_span())),
		}
	}

	pub fn read_maybe<V, F: FnOnce(&mut Self, Token, Span) -> WithToken<V>>(
		&mut self,
		consumer: F,
	) -> Option<V> {
		let (token, span) = self.read_next();
		match consumer(self, token, span) {
			WithToken::Read(value) => Some(value),
			WithToken::None(token) => {
				self.unget(token, span);
				None
			}
		}
	}

	pub fn read_with<V, F: FnOnce(Token, Span) -> Option<V>>(&mut self, consumer: F) -> Option<V> {
		let (token, span) = self.read_next();
		consumer(token, span)
	}

	pub fn read<V, F: FnOnce(&mut Self, Token, Span) -> Option<V>>(
		&mut self,
		consumer: F,
	) -> Option<V> {
		let (token, span) = self.read_next();
		if let Token::None = token {
			None
		} else {
			consumer(self, token, span)
		}
	}

	pub fn read_map<V, F: FnOnce(Token, Span) -> V>(&mut self, consumer: F) -> V {
		let (token, span) = self.read_next();
		consumer(token, span)
	}

	pub fn read_if<F: Fn(&Token) -> bool>(&mut self, predicate: F) -> bool {
		let (token, _) = self.peek_next();
		if predicate(token) {
			self.next.pop_front();
			true
		} else {
			false
		}
	}

	pub fn skip_while<F: Fn(&Token) -> bool>(&mut self, predicate: F) {
		loop {
			match self.peek_next() {
				(Token::None, _) => break,
				(token, _) => {
					if predicate(token) {
						self.next.pop_front();
					} else {
						break;
					}
				}
			}
		}
	}

	pub fn next_span(&mut self) -> Span {
		self.peek_next().1
	}

	pub fn next_token(&mut self) -> &Token {
		&self.peek_next().0
	}

	pub fn dump_next(&mut self) -> (Token, Span, &str) {
		let (token, span) = self.read_next();
		let text = self.input.read_text(span);
		(token, span, text)
	}

	/// Returns a child `TokenStream` that will iterate tokens from the
	/// current one, but will stop at a [`Token::Ident`].
	#[allow(unused)]
	pub fn indented(&mut self) -> TokenStream<T> {
		todo!()
	}

	/// Returns a child `TokenStream` that will iterate tokens from the
	/// current one, but will stop at tokens for which `f` returns true.
	#[allow(unused)]
	pub fn until(&mut self, f: fn(Token) -> bool) -> TokenStream<T> {
		todo!()
	}

	/// Returns a child `TokenStream` that will iterate tokens from the
	/// current one, but will stop at the given right parenthesis.
	///
	/// Note that this overrides an [`TokenStream::until`] limitation.
	#[allow(unused)]
	pub fn parenthesized(&mut self, right: Token) -> TokenStream<T> {
		todo!()
	}

	/// Returns a token to the stream, making it available for reading again.
	pub fn unget(&mut self, token: Token, span: Span) {
		let next = self.next_span();
		assert!(span.end.offset <= next.pos.offset);
		self.next.push_front((token, span));
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
