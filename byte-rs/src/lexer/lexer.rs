use std::cell::RefCell;

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
	next: RefCell<Option<(TokenAt, State)>>,
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
			next: RefCell::new(None),
		}
	}

	pub fn config<F: FnOnce(&mut Scanner)>(&mut self, config: F) {
		self.next.replace(None);
		self.state.stream.config(config);
	}

	pub fn errors(&self) -> ErrorList {
		self.state.stream.errors().clone()
	}

	pub fn next(&self) -> TokenAt {
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

	pub fn read(&mut self) -> TokenAt {
		if let Some((token, state)) = self.next.take() {
			self.state = state;
			return token;
		}

		let empty = self.pos().col() == 0;
		loop {
			let state = &mut self.state;
			state.stream.skip();
			let start = state.stream.pos();
			let errors = state.stream.errors_mut();
			let token = if let Some(token) = state.indent.check_indent(&start, errors) {
				token
			} else {
				state.stream.read()
			};
			if token.is::<Comment>() || (empty && token == Token::Break) {
				continue;
			}

			let span = Span {
				sta: start.clone(),
				end: state.stream.pos(),
			};
			break TokenAt(span, token);
		}
	}
}

impl Stream for Lexer {
	fn pos(&self) -> Cursor {
		self.state.stream.pos()
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
