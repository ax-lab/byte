use std::{
	cell::{Cell, RefCell},
	rc::Rc,
};

use crate::{Error, Result};

use super::{Config, Cursor, Indent, Input, Lex, LexerResult, Matcher, Span, Token};

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
	state: Rc<RefCell<State<'a>>>,
	config: Rc<Config>,
	index: usize,
	current: Cell<Option<Lex<'a>>>,
	errors: RefCell<Rc<Vec<Error<'a>>>>,
}

impl<'a> Context<'a> {
	pub fn new(source: &'a dyn Input, config: Config) -> Self {
		let state = State {
			source,
			entries: Vec::new(),
		};
		let out = Context {
			state: Rc::new(state.into()),
			config: Rc::new(config),
			index: 0,
			current: Cell::new(None),
			errors: Default::default(),
		};
		out
	}

	pub fn errors(&self) -> Vec<Error<'a>> {
		let errors = self.errors.borrow();
		(**errors).clone()
	}

	pub fn value(&self) -> Lex<'a> {
		match self.current.get() {
			Some(value) => value,
			None => {
				let mut state = self.state.borrow_mut();
				let value = state.get_index(&self.config, self.index);
				let value = match value {
					Ok(value) => value,
					Err(error) => {
						let span = error.span();
						let mut errors = self.errors.borrow_mut();
						let errors = Rc::make_mut(&mut errors);
						errors.push(error);
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

	pub fn source(&self) -> &'a dyn Input {
		self.state.borrow().source
	}

	pub fn next(&mut self) {
		if !self.token().is_none() {
			self.index += 1;
			self.current.set(None);
		}
	}

	pub fn token(&self) -> Token {
		self.value().token
	}

	pub fn span(&self) -> Span<'a> {
		self.value().span
	}

	//----[ Reader helpers ]--------------------------------------------------//

	/// Return the next token and true if the predicate matches the current
	/// token.
	pub fn next_if<F: Fn(Lex) -> bool>(&mut self, predicate: F) -> bool {
		if predicate(self.value()) {
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
struct Entry<'a> {
	token: Token,
	span: Span<'a>,
	prev: Option<usize>,
	head: Option<usize>,
}

#[derive(Clone)]
struct State<'a> {
	entries: Vec<Entry<'a>>,
	source: &'a dyn Input,
}

impl<'a> State<'a> {
	pub fn pos(&self) -> Cursor<'a> {
		self.entries
			.last()
			.map(|x| x.span.end)
			.unwrap_or(Cursor::new(self.source))
	}

	pub fn head(&self) -> Option<usize> {
		self.entries.last().map(|x| x.head).unwrap_or_default()
	}

	pub fn get_index(&mut self, config: &Config, index: usize) -> Result<'a, Lex<'a>> {
		while index >= self.entries.len() {
			if !self.fill_next(config)? {
				let token = Token::None;
				let pos = self.pos();
				let span = Span { pos: pos, end: pos };
				return Ok(Lex { token, span });
			}
		}
		Ok(Lex {
			token: self.entries[index].token,
			span: self.entries[index].span,
		})
	}

	fn fill_next(&mut self, config: &Config) -> Result<'a, bool> {
		let start_count = self.entries.len();
		let mut cursor = self.pos();
		let empty = cursor.column == 0;
		loop {
			let new_line = cursor.column == 0;
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
						pos: start,
						end: span.pos,
					};
					self.indent(span);
				} else {
					while indent < self.indent_level() {
						self.dedent(Span {
							pos: start,
							end: span.pos,
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

	fn indent(&mut self, span: Span<'a>) {
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
				return head.span.end.column;
			}
		}
		0
	}

	fn dedent(&mut self, span: Span<'a>) -> Result<'a, ()> {
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

	fn open_paren(&mut self, token: Token, span: Span<'a>) {
		let head = self.head();
		self.entries.push(Entry {
			token,
			span,
			prev: head,
			head: Some(self.entries.len()),
		});
	}

	fn close_paren(
		&mut self,
		token: Token,
		span: Span<'a>,
		symbol: &'static str,
	) -> Result<'a, ()> {
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
					if self.indent_level() > span.pos.column {
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
