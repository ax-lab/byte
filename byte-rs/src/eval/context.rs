use crate::{
	input::Input,
	lexer::{Cursor, Lex, Stream, Token},
	Error,
};

use super::Node;

#[derive(Clone)]
pub struct Context<'a> {
	input: Stream<'a>,
}

impl<'a> Context<'a> {
	pub fn new(input: Stream<'a>) -> Self {
		Context { input }
	}

	pub fn finish(self, program: Vec<Node>) -> (Vec<Node>, Vec<Error<'a>>) {
		(program, self.input.errors())
	}

	pub fn is_valid(&self) -> bool {
		true
	}
}

// Lexing
impl<'a> Context<'a> {
	pub fn pos(&self) -> Cursor<'a> {
		self.input.pos()
	}

	pub fn source(&self) -> &'a dyn Input {
		self.input.source()
	}

	pub fn lex(&self) -> Lex<'a> {
		self.input.value()
	}

	pub fn token(&self) -> Token {
		self.input.token()
	}

	pub fn next(&mut self) {
		self.input.next()
	}

	pub fn has_some(&self) -> bool {
		self.lex().is_some()
	}
}

// Parsing helpers
impl<'a> Context<'a> {
	pub fn check_end(&mut self) -> bool {
		if self.has_some() {
			self.input.add_error(Error::ExpectedEnd(self.lex()));
			false
		} else {
			true
		}
	}
}
