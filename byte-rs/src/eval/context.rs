use std::collections::VecDeque;

use crate::{
	input::Input,
	lexer::{Cursor, Lex, Span, Stream, Token},
	Error,
};

use super::{
	macros::{self, Macro},
	Node,
};

#[derive(Copy, Clone)]
enum Scope {
	Root,
	Line(Option<&'static str>),
}

#[derive(Clone)]
pub struct Context<'a> {
	input: Stream<'a>,
	scope: VecDeque<Scope>,
}

impl<'a> Context<'a> {
	pub fn new(input: Stream<'a>) -> Self {
		Context {
			input,
			scope: Default::default(),
		}
	}

	pub fn add_error(&mut self, error: Error<'a>) {
		self.input.add_error(error);
	}

	pub fn finish(self, program: Vec<Node>) -> (Vec<Node>, Vec<Error<'a>>) {
		(program, self.input.errors())
	}

	pub fn is_valid(&self) -> bool {
		!self.input.has_errors()
	}

	fn scope(&self) -> Scope {
		self.scope.front().copied().unwrap_or(Scope::Root)
	}

	#[allow(unused)]
	pub fn scope_line(mut self) -> Self {
		self.scope.push_front(Scope::Line(None));
		self
	}

	pub fn scope_line_with_break(mut self, with_break: &'static str) -> Self {
		self.scope.push_front(Scope::Line(Some(with_break)));
		self
	}

	pub fn pop_scope(mut self) -> Self {
		self.scope.pop_front().expect("no scope to pop");
		self
	}

	pub fn get_macro(&self, name: &str) -> Option<Box<dyn Macro>> {
		if name == "add1" {
			Some(Box::new(macros::AddOne))
		} else {
			None
		}
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
		let next = self.input.value();
		let next = match self.scope() {
			Scope::Root => next,
			Scope::Line(with_break) => match next.token {
				Token::Symbol(sym) if Some(sym) == with_break => next.as_none(),
				Token::Break => next.as_none(),
				_ => next,
			},
		};
		next
	}

	pub fn span(&self) -> Span<'a> {
		self.lex().span
	}

	pub fn next(&mut self) {
		if self.lex().is_some() {
			self.input.next();
		}
	}

	pub fn token(&self) -> Token {
		self.lex().token
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

	pub fn skip_symbol(&mut self, symbol: &str) -> bool {
		if !self.lex().is_some() {
			false
		} else {
			self.input.skip_symbol(symbol)
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::lexer;

	#[test]
	fn root_scope() {
		let input = "+ -";
		let mut context = Context::new(lexer::open(&input));

		assert_eq!(context.token(), Token::Symbol("+"));
		context.next();

		assert_eq!(context.token(), Token::Symbol("-"));
		context.next();

		assert_eq!(context.token(), Token::None);
	}

	#[test]
	fn line_scope() {
		let input = "+\n-\n*\n.";
		let context = Context::new(lexer::open(&input));

		// line scope should stop at the line break
		let mut sub = context.scope_line();
		assert_eq!(sub.token(), Token::Symbol("+"));
		sub.next();
		assert_eq!(sub.token(), Token::None);
		sub.next();
		assert_eq!(sub.token(), Token::None);
		sub.next();

		// once the scope is back to root, the break can be read
		let mut context = sub.pop_scope();
		assert_eq!(context.token(), Token::Break);
		context.next();

		// next line scope
		let mut sub = context.scope_line();
		assert_eq!(sub.token(), Token::Symbol("-"));
		sub.next();
		assert_eq!(sub.token(), Token::None);

		// back to root
		let mut context = sub.pop_scope();
		assert_eq!(context.token(), Token::Break);
		context.next();
		assert_eq!(context.token(), Token::Symbol("*"));
		context.next();
		assert_eq!(context.token(), Token::Break);

		// create a line scope right at the end
		let sub = context.scope_line();
		assert_eq!(sub.token(), Token::None);

		// and pop it
		let mut context = sub.pop_scope();
		assert_eq!(context.token(), Token::Break);
		context.next();
		assert_eq!(context.token(), Token::Symbol("."));
		context.next();
		assert_eq!(context.token(), Token::None);
	}

	#[test]
	fn line_scope_with_break() {
		let input = "+;-\n1; 2";
		let context = Context::new(lexer::open(&input));

		// line scope should stop at the line break
		let mut sub = context.scope_line_with_break(";");
		assert_eq!(sub.token(), Token::Symbol("+"));
		sub.next();
		assert_eq!(sub.token(), Token::None);

		// once the scope is back to root, the break can be read
		let mut context = sub.pop_scope();
		assert_eq!(context.token(), Token::Symbol(";"));
		context.next();

		// next line scope
		let mut sub = context.scope_line_with_break(";");
		assert_eq!(sub.token(), Token::Symbol("-"));
		sub.next();
		assert_eq!(sub.token(), Token::None);

		// back to root
		let mut context = sub.pop_scope();
		assert_eq!(context.token(), Token::Break);
		context.next();

		let mut sub = context.scope_line_with_break(";");
		assert_eq!(sub.token(), Token::Integer(1));
		sub.next();
		assert_eq!(sub.token(), Token::None);

		let mut context = sub.pop_scope();
		assert_eq!(context.token(), Token::Symbol(";"));
		context.next();
		assert_eq!(context.token(), Token::Integer(2));
		context.next();
		assert_eq!(context.token(), Token::None);
	}
}
