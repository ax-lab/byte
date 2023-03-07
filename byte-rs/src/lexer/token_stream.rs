use std::{collections::VecDeque, rc::Rc};

use super::{Input, LexerResult, Reader, Span, Token};

pub struct TokenStream {
	reader: Reader,
	tokens: Rc<Vec<(Token, Span)>>,
	index: usize,
}

pub enum ReadToken<T> {
	MapTo(T),
	Unget(Token),
}

impl TokenStream {
	pub fn new<T: Input + 'static>(input: T) -> TokenStream {
		let mut reader = Reader::from(input);
		let tokens = read_all(&mut reader);
		TokenStream {
			reader,
			tokens: Rc::new(tokens),
			index: 0,
		}
	}

	pub fn read_text(&self, pos: usize, end: usize) -> String {
		self.reader.read_text(pos, end)
	}

	/// Span for the next token in the input for use in compiler messages.
	pub fn next_span(&mut self) -> Span {
		self.tokens[self.index].1
	}

	/// Next token in the input for use in compiler messages.
	pub fn next_token(&mut self) -> (Token, Span) {
		self.tokens[self.index].clone()
	}

	/// Read the next token in the input and passes it to the given parser
	/// function returning an [`Option`] result.
	///
	/// At the end of the input, returns [`None`] without calling the parser.
	pub fn read<V, F: FnOnce(&mut Self, Token, Span) -> Option<V>>(
		&mut self,
		parser: F,
	) -> Option<V> {
		let (token, span) = self.read_pair();
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
		let (token, span) = self.read_pair();
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
		let (token, span) = self.read_pair();
		parser(token, span)
	}

	/// Similar to `map_next` but does not consume the token if the mapping
	/// function returns [`None`].
	pub fn try_map_next<V, F: FnOnce(&Token, Span) -> Option<V>>(&mut self, map: F) -> Option<V> {
		let value = {
			let next = &self.tokens[self.index];
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
			let next = &self.tokens[self.index];
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
			let next = &self.tokens[self.index];
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
	pub fn unget(&mut self, _token: Token, span: Span) {
		assert!(self.index > 0);
		self.index -= 1;
		assert!(self.next_span() == span);
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

	pub fn skip_blank_lines(&mut self) {
		self.skip_while(|token| matches!(token, Token::LineBreak | Token::Comment));
	}

	//------------------------------------------------------------------------//
	// Internal methods
	//------------------------------------------------------------------------//

	fn shift(&mut self) {
		if self.index < self.tokens.len() - 1 {
			self.index += 1;
		}
	}

	pub fn read_pair(&mut self) -> (Token, Span) {
		let out = self.tokens[self.index].clone();
		self.shift();
		out
	}
}

fn read_all(input: &mut Reader) -> Vec<(Token, Span)> {
	let mut tokens = Vec::new();
	let mut indent = VecDeque::new();
	let mut parens = VecDeque::new();

	let mut is_last = false;
	while !is_last {
		// check if we are at the start of the line so we can compute indentation
		let start = input.pos();
		let new_line = start.column == 0;

		// read the next token
		let (result, span) = super::read_token(input);
		let token = match result {
			LexerResult::Token(token) => token,
			LexerResult::None => Token::None,
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

		let need_indent =
			(new_line && token != Token::LineBreak) || token == Token::None || closing;
		let column = if token == Token::None {
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

		is_last = token == Token::None;
		if token != Token::Comment {
			tokens.push((token, span));
		}
	}
	tokens
}
