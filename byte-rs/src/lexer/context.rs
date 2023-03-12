use std::{cell::RefCell, rc::Rc};

use super::{Cursor, Indent, Input, Lex, LexerResult, Span, Token};

/// Holds the lexer state at a particular point in the input and provides
/// methods for consuming tokens.
///
/// Can be cloned with little overhead providing the ability to save and
/// backtrack to a previous lexer state.
///
/// The context can also be configured during the lexing process, which will
/// take effect going forward in the lexing.
#[derive(Clone)]
pub struct Context<'a> {
	pub value: Lex<'a>,
	state: Rc<RefCell<State<'a>>>,
	index: usize,
}

impl<'a> Context<'a> {
	pub fn new(source: &'a dyn Input) -> Self {
		let cursor = Cursor::new(source);
		let mut state = State {
			tokens: Vec::new(),
			cursor,
			last: 0,
		};
		let out = Context {
			value: state.get_index(0),
			state: Rc::new(state.into()),
			index: 0,
		};
		out
	}

	pub fn source(&self) -> &'a dyn Input {
		self.state.borrow().cursor.source
	}

	pub fn token(&self) -> Token {
		self.value.token
	}

	pub fn span(&self) -> Span<'a> {
		self.value.span
	}

	pub fn triple(&self) -> (Token, Span<'a>, &str) {
		(self.value.token, self.value.span, self.value.span.text())
	}

	pub fn next(&mut self) {
		let mut state = self.state.borrow_mut();
		if !self.value.token.is_none() {
			self.index += 1;
			self.value = state.get_index(self.index);
		}
	}

	/// Return the next token and true if the predicate matches the current
	/// token.
	pub fn next_if<F: Fn(Lex) -> bool>(&mut self, predicate: F) -> bool {
		if predicate(self.value) {
			self.next();
			true
		} else {
			false
		}
	}

	/// Read the next token if it is the specific symbol.
	pub fn skip_symbol(&mut self, symbol: &str) -> bool {
		self.next_if(|value| value.symbol() == Some(symbol))
	}
}

struct State<'a> {
	pub tokens: Vec<(Token, Span<'a>, usize)>,
	cursor: Cursor<'a>,
	last: usize,
}

impl<'a> State<'a> {
	pub fn get_index(&mut self, index: usize) -> Lex<'a> {
		while index >= self.tokens.len() {
			if !self.fill_next() {
				let token = Token::None;
				let span = Span {
					pos: self.cursor,
					end: self.cursor,
				};
				return Lex { token, span };
			}
		}
		Lex {
			token: self.tokens[index].0,
			span: self.tokens[index].1,
		}
	}

	fn fill_next(&mut self) -> bool {
		let start_count = self.tokens.len();
		let empty = self.cursor.column == 0;
		loop {
			let new_line = self.cursor.column == 0;
			let start = self.cursor;
			let input = &mut self.cursor;

			// read the next token
			let (result, span) = super::read_token(input);
			let (token, end, indent) = match result {
				LexerResult::Token(token, Indent(indent)) => (token, false, indent),
				LexerResult::None => (Token::Break, true, 0),
				LexerResult::Error(error) => panic!("{error} at {span}"),
			};

			if let Some(symbol) = token.is_closing() {
				self.close_paren(token, span, symbol);
				return true;
			}

			let need_indent = (new_line && token != Token::Break) || end;

			// check if we need indent or dedent tokens by comparing the first token level
			if need_indent {
				let level = self.indent_level();
				if indent > level {
					let span = Span {
						pos: start,
						end: span.pos,
					};
					self.indent(span);
				} else {
					while indent < self.indent_level() {
						self.dedent(Span {
							pos: start,
							end: span.pos,
						});
					}
				}
			}

			if let Some(_) = token.get_closing() {
				self.open_paren(token, span);
			} else {
				let skip = match token {
					Token::Break => new_line || empty || end,
					_ => false,
				};

				if !skip {
					self.tokens.push((token, span, 0));
					break;
				}
			}

			if end {
				break;
			}
		}
		self.tokens.len() > start_count
	}

	fn indent(&mut self, span: Span<'a>) {
		self.tokens.push((Token::Indent, span, self.last));
		self.last = self.tokens.len();
	}

	fn indent_level(&self) -> usize {
		let mut last = self.last;
		while last > 0 {
			let previous = &self.tokens[last - 1];
			last = previous.2;
			if let Token::Indent = previous.0 {
				return previous.1.end.column;
			}
		}
		0
	}

	fn dedent(&mut self, span: Span<'a>) {
		let expected = if self.last > 0 {
			let previous = &self.tokens[self.last - 1];
			if let Token::Indent = previous.0 {
				self.last = previous.2;
				self.tokens.push((Token::Dedent, span, 0));
				true
			} else {
				false
			}
		} else {
			false
		};
		if !expected {
			panic!("error: unexpected Dedent at {span}");
		}
	}

	fn open_paren(&mut self, token: Token, span: Span<'a>) {
		self.tokens.push((token, span, self.last));
		self.last = self.tokens.len();
	}

	fn close_paren(&mut self, token: Token, span: Span<'a>, symbol: &'static str) {
		while self.last > 0 {
			let previous = &self.tokens[self.last - 1];
			self.last = previous.2;

			match previous.0 {
				Token::Indent => {
					self.tokens.push((Token::Dedent, span, 0));
				}

				left if left.get_closing() == Some(symbol) => {
					self.tokens.push((token, span, 0));
					if self.indent_level() > span.pos.column {
						panic!("error: unexpected Dedent before {symbol} at {span}");
					}
					break;
				}

				_ => {
					panic!("error: unexpected closing {symbol} at {span}");
				}
			}
		}
	}
}
