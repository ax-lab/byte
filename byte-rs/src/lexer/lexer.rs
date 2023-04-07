use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

use crate::core::error::*;
use crate::core::input::*;

use super::*;

/// [`Lexer`] holds the entire lexing state and configuration for a position
/// in the input, given by a [`Cursor`].
///
/// The lexer output is a stream of [`Token`] ready to be parsed.
///
/// Cloning a [`Lexer`] is a low-overhead operation and allows saving an
/// input position and configuration state, allowing a parser to backtrack
/// fully.
///
/// It is the lexer's responsibility to apply high-level language tokenization
/// semantics such as indentation (i.e. [`Token::Indent`] and [`Token::Dedent`])
/// and filtering of ignored tokens such as [`Comment`].
///
/// The lexer can be reconfigured during parsing, either from the input source
/// text or by the parser itself.
#[derive(Clone)]
pub struct Lexer {
	state: State,
	next: RefCell<Rc<VecDeque<(TokenAt, State)>>>,
}

#[derive(Clone)]
struct State {
	stream: TokenStream,
	indent: Indent,
}

impl Lexer {
	pub fn new(input: Cursor, scanner: Scanner) -> Self {
		Lexer {
			state: State {
				stream: TokenStream::new(input, scanner),
				indent: Indent::new(),
			},
			next: RefCell::new(Rc::new(VecDeque::new())),
		}
	}

	#[allow(unused)]
	pub fn start_indent(&mut self) -> IndentRegion {
		self.state.indent.open_region()
	}

	#[allow(unused)]
	pub fn end_indent(&mut self, region: IndentRegion) {
		self.state.indent.close_region(region);
	}

	pub fn config<F: FnOnce(&mut Scanner)>(&mut self, config: F) {
		let mut next = self.next.borrow_mut();
		*next = Rc::new(VecDeque::new());
		self.state.stream.config(config);
	}

	pub fn errors(&self) -> ErrorList {
		self.state.stream.errors().clone()
	}

	pub fn lookahead(&self, n: usize) -> TokenAt {
		{
			let next = self.next.borrow();
			if let Some((token, ..)) = next.get(n) {
				return token.clone();
			} else if let Some((last, ..)) = next.back() {
				if last.is_none() {
					return last.clone();
				}
			}
		}

		let mut next = self.next.borrow_mut();
		let next = Rc::make_mut(&mut next);
		let mut state = next
			.back()
			.map(|x| x.1.clone())
			.unwrap_or_else(|| self.state.clone());
		while n >= next.len() {
			let token = state.read();
			let is_none = token.is_none();
			next.push_back((token.clone(), state.clone()));
			if is_none {
				break;
			}
		}
		next.back().map(|x| x.0.clone()).unwrap()
	}

	pub fn next(&self) -> TokenAt {
		self.lookahead(0)
	}

	pub fn read(&mut self) -> TokenAt {
		let mut next = self.next.borrow_mut();
		let next = Rc::make_mut(&mut next);
		if let Some((token, state)) = next.pop_front() {
			self.state = state;
			return token;
		} else {
			self.state.read()
		}
	}
}

impl State {
	fn read(&mut self) -> TokenAt {
		let empty = self.stream.pos().col() == 0;
		loop {
			self.stream.skip();
			let start = self.stream.pos().clone();
			let errors = self.stream.errors_mut();
			let next = if let Some(next) = self.indent.check_indent(&start, errors) {
				next
			} else {
				self.stream.read()
			};
			let token = next.token();
			if token.is::<Comment>() || (empty && token == Token::Break) {
				continue;
			}
			break next;
		}
	}
}

impl Stream for Lexer {
	fn pos(&self) -> Cursor {
		self.state.stream.pos().clone()
	}

	fn copy(&self) -> Box<dyn Stream> {
		Box::new(self.clone())
	}

	fn next(&self) -> TokenAt {
		Lexer::next(self)
	}

	fn read(&mut self) -> TokenAt {
		Lexer::read(self)
	}

	fn errors(&self) -> ErrorList {
		self.state.stream.errors().clone()
	}

	fn add_error(&mut self, error: Error) {
		self.state.stream.errors_mut().add(error)
	}
}
