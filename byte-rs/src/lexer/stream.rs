use std::{
	cell::{Cell, RefCell},
	rc::Rc,
};

use crate::input::*;

use crate::{Error, Result};

use super::{Config, Indent, Input, Lex, LexerResult, Matcher, Token};

pub trait LexStream {
	fn copy(&self) -> Box<dyn LexStream>;

	fn source(&self) -> Input;

	fn next(&self) -> Lex;
	fn advance(&mut self);

	fn add_error(&mut self, error: Error);
	fn errors(&self) -> Vec<Error>;
	fn has_errors(&self) -> bool;

	fn token(&self) -> Token {
		self.next().token
	}

	fn span(&self) -> Span {
		self.next().span
	}

	fn peek_after(&self) -> Lex {
		let mut input = self.copy();
		input.advance();
		input.next()
	}

	//----[ Reader helpers ]--------------------------------------------------//

	fn at_end(&self) -> bool {
		!self.has_some()
	}

	fn has_some(&self) -> bool {
		self.next().is_some()
	}

	fn pos(&self) -> Cursor {
		self.next().span.sta
	}

	fn from(&self, pos: Cursor) -> Span {
		Span {
			sta: pos,
			end: self.pos(),
		}
	}

	/// Return the next token and true if the predicate matches the current
	/// token.
	fn next_if(&mut self, predicate: &dyn Fn(Lex) -> bool) -> bool {
		if predicate(self.next()) {
			self.advance();
			true
		} else {
			false
		}
	}

	/// Read the next token if it is the specific symbol.
	fn skip_symbol(&mut self, symbol: &str) -> bool {
		self.next_if(&|value| value.symbol() == Some(symbol))
	}

	fn check_end(&mut self) -> bool {
		if self.has_some() {
			self.add_error(Error::ExpectedEnd(self.next()));
			false
		} else {
			true
		}
	}
}

/// Holds the lexer state at a particular point in the input and provides
/// methods for consuming tokens.
///
/// Can be cloned with little overhead providing the ability to save and
/// backtrack to a previous lexer state.
///
/// The context can also be configured during the lexing process, which will
/// take effect going forward in the lexing.
#[derive(Clone)]
pub struct Stream {
	state: Rc<RefCell<State>>,
	config: Rc<Config>,
	index: usize,

	current: Cell<Option<Lex>>,
	current_error: RefCell<Option<Error>>,

	errors: RefCell<Rc<Vec<Error>>>,
}

impl LexStream for Stream {
	fn copy(&self) -> Box<dyn LexStream> {
		Box::new(self.clone())
	}

	fn has_errors(&self) -> bool {
		self.errors.borrow().len() > 0 || self.current_error.borrow().is_some()
	}

	fn errors(&self) -> Vec<Error> {
		let errors = self.errors.borrow();
		let mut errors = (**errors).clone();
		if let Some(error) = &*self.current_error.borrow() {
			errors.push(error.clone());
		}
		errors
	}

	fn add_error(&mut self, error: Error) {
		let mut errors = self.errors.borrow_mut();
		let errors = Rc::make_mut(&mut errors);
		errors.push(error);
	}

	fn next(&self) -> Lex {
		match self.current.get() {
			Some(value) => value,
			None => {
				let mut state = self.state.borrow_mut();
				let value = state.get_index(&self.config, self.index);
				let value = match value {
					Ok(value) => value,
					Err(error) => {
						let span = error.span();
						let old_error = self.current_error.replace(Some(error));
						assert!(old_error.is_none());
						Lex {
							token: Token::Invalid,
							span,
						}
					}
				};
				self.current.set(Some(value));
				value
			}
		}
	}

	fn source(&self) -> Input {
		self.state.borrow().source
	}

	fn advance(&mut self) {
		if !self.token().is_none() {
			self.index += 1;
			self.current.set(None);
			if let Some(error) = self.current_error.replace(None) {
				self.add_error(error);
			}
		}
	}
}

impl Stream {
	pub fn new(source: Input, config: Config) -> Self {
		let state = State {
			source,
			entries: Vec::new(),
		};
		let out = Stream {
			state: Rc::new(state.into()),
			config: Rc::new(config),
			index: 0,
			current: Cell::new(None),
			current_error: RefCell::new(None),
			errors: Default::default(),
		};
		out
	}

	//----[ Configuration ]---------------------------------------------------//

	#[allow(unused)]
	pub fn add_symbol(&mut self, symbol: &'static str, token: Token) {
		Rc::make_mut(&mut self.config).add_symbol(symbol, token);
		self.trim_state();
	}

	#[allow(unused)]
	pub fn add_matcher(&mut self, matcher: Box<dyn Matcher>) {
		Rc::make_mut(&mut self.config).add_matcher(matcher);
		self.trim_state();
	}

	fn trim_state(&mut self) {
		let new_length = self.index;
		self.current.set(None);
		self.current_error.replace(None);
		if Rc::strong_count(&self.state) > 1 {
			let mut new_state = self.state.borrow().clone();
			new_state.entries.truncate(new_length);
			self.state = Rc::new(new_state.into());
		} else {
			let mut state = self.state.borrow_mut();
			state.entries.truncate(new_length);
		}
	}
}

//----------------------------------------------------------------------------//
// State
//----------------------------------------------------------------------------//

#[derive(Clone)]
struct Entry {
	token: Token,
	span: Span,
	prev: Option<usize>,
	head: Option<usize>,
}

#[derive(Clone)]
struct State {
	entries: Vec<Entry>,
	source: Input,
}

impl State {
	pub fn cur(&self) -> Cursor {
		self.entries
			.last()
			.map(|x| x.span.end)
			.unwrap_or(self.source.sta())
	}

	pub fn head(&self) -> Option<usize> {
		self.entries.last().map(|x| x.head).unwrap_or_default()
	}

	pub fn get_index(&mut self, config: &Config, index: usize) -> Result<Lex> {
		while index >= self.entries.len() {
			if !self.fill_next(config)? {
				let token = Token::None;
				let pos = self.cur();
				let span = Span { sta: pos, end: pos };
				return Ok(Lex { token, span });
			}
		}
		Ok(Lex {
			token: self.entries[index].token,
			span: self.entries[index].span,
		})
	}

	fn fill_next(&mut self, config: &Config) -> Result<bool> {
		let start_count = self.entries.len();
		let mut cursor = self.cur();
		let empty = cursor.col() == 0;
		loop {
			let new_line = cursor.col() == 0;
			let start = cursor;
			let input = &mut cursor;

			// read the next token
			let (result, span) = super::read_token(config, input);
			let (token, end, indent) = match result {
				LexerResult::Token(token, Indent(indent)) => (token, false, indent),
				LexerResult::None => (Token::Break, true, 0),
				LexerResult::Error(error) => return Error::Lexer(error, span).into(),
			};

			if let Some(symbol) = token.is_closing() {
				self.close_paren(token, span, symbol)?;
				return Ok(true);
			}

			let need_indent = (new_line && token != Token::Break) || end;

			// check if we need indent or dedent tokens by comparing the first token level
			if need_indent {
				let level = self.indent_level();
				if indent > level {
					let span = Span {
						sta: start,
						end: span.sta,
					};
					self.indent(span);
				} else {
					while indent < self.indent_level() {
						self.dedent(Span {
							sta: start,
							end: span.sta,
						})?;
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
					let head = self.head().map(|x| &self.entries[x]);
					self.entries.push(Entry {
						token,
						span,
						prev: head.map(|x| x.prev).unwrap_or_default(),
						head: head.map(|x| x.head).unwrap_or_default(),
					});
					break;
				}
			}

			if end {
				break;
			}
		}
		Ok(self.entries.len() > start_count)
	}

	fn indent(&mut self, span: Span) {
		self.entries.push(Entry {
			token: Token::Indent,
			span,
			prev: self.head(),
			head: Some(self.entries.len()),
		});
	}

	fn indent_level(&self) -> usize {
		let mut current = self.head();
		while let Some(index) = current {
			let head = &self.entries[index];
			current = head.prev;
			if let Token::Indent = head.token {
				return head.span.end.col();
			}
		}
		0
	}

	fn dedent(&mut self, span: Span) -> Result<()> {
		let expected = if let Some(index) = self.head() {
			let head = &self.entries[index];
			if let Token::Indent = head.token {
				self.entries.push(Entry {
					token: Token::Dedent,
					span,
					prev: head.prev.map(|x| self.entries[x].prev).unwrap_or_default(),
					head: head.prev,
				});
				true
			} else {
				false
			}
		} else {
			false
		};
		if !expected {
			Error::Dedent(span).into()
		} else {
			Ok(())
		}
	}

	fn open_paren(&mut self, token: Token, span: Span) {
		let head = self.head();
		self.entries.push(Entry {
			token,
			span,
			prev: head,
			head: Some(self.entries.len()),
		});
	}

	fn close_paren(&mut self, token: Token, span: Span, symbol: &'static str) -> Result<()> {
		let mut current = self.head();
		while let Some(index) = current {
			let head = &self.entries[index];
			current = head.prev;

			match head.token {
				Token::Indent => {
					self.entries.push(Entry {
						token: Token::Dedent,
						span,
						prev: current.map(|x| self.entries[x].prev).unwrap_or_default(),
						head: head.prev,
					});
				}

				left if left.get_closing() == Some(symbol) => {
					self.entries.push(Entry {
						token,
						span,
						prev: current.map(|x| self.entries[x].prev).unwrap_or_default(),
						head: head.prev,
					});
					if self.indent_level() > span.sta.col() {
						return Error::ClosingDedent(symbol, span).into();
					}
					break;
				}

				_ => {
					return Error::ClosingSymbol(symbol, span).into();
				}
			}
		}
		Ok(())
	}
}
