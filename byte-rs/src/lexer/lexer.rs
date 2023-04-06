use std::cell::RefCell;
use std::rc::Rc;

use crate::core::error::*;
use crate::core::input::*;

use super::*;

/// Holds all the lexer state and provides lexing for tokens.
#[derive(Clone)]
pub struct Lexer {
	scanner: Rc<Scanner>,
	state: State,
	next: RefCell<Option<(Lex, State)>>,
}

#[derive(Clone)]
struct State {
	input: Cursor,
	indent: Indent,
	errors: ErrorList,
}

impl Lexer {
	pub fn new(input: Cursor, scanner: Scanner) -> Self {
		Lexer {
			scanner: Rc::new(scanner),
			state: State {
				input,
				indent: Indent::new(),
				errors: ErrorList::new(),
			},
			next: RefCell::new(None),
		}
	}

	pub fn config<F: FnOnce(&mut Scanner)>(&mut self, config: F) {
		self.next.replace(None);
		let scanner = Rc::make_mut(&mut self.scanner);
		config(scanner)
	}

	pub fn errors(&self) -> ErrorList {
		self.state.errors.clone()
	}

	pub fn next(&self) -> Lex {
		let token = {
			let next = self.next.borrow();
			if let Some((token, ..)) = &*next {
				Some(token.clone())
			} else {
				None
			}
		};
		if let Some(token) = token {
			token
		} else {
			let mut clone = self.clone();
			let next = clone.read();
			self.next.replace(Some((next.clone(), clone.state)));
			next
		}
	}

	pub fn read(&mut self) -> Lex {
		if let Some((token, state)) = self.next.take() {
			self.state = state;
			return token;
		}

		let empty = self.state.input.col() == 0;
		loop {
			let state = &mut self.state;
			self.scanner.skip(&mut state.input);
			let start = state.input.clone();
			let token =
				if let Some(token) = state.indent.check_indent(&state.input, &mut state.errors) {
					token
				} else {
					self.scanner.read(&mut state.input, &mut state.errors)
				};
			if token.is::<Comment>() || (empty && token == Token::Break) {
				continue;
			}

			let span = Span {
				sta: start.clone(),
				end: state.input.clone(),
			};
			break Lex(span, token);
		}
	}
}

impl Stream for Lexer {
	fn pos(&self) -> Cursor {
		self.state.input.clone()
	}

	fn copy(&self) -> Box<dyn Stream> {
		Box::new(self.clone())
	}

	fn next(&self) -> Lex {
		Lexer::next(self)
	}

	fn read(&mut self) -> Lex {
		Lexer::read(self)
	}

	fn errors(&self) -> ErrorList {
		self.state.errors.clone()
	}

	fn add_error(&mut self, error: Error) {
		self.state.errors.add(error)
	}
}
