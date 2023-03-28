use crate::{
	input::Input,
	lexer::{Lex, LexStream, Stream},
	Error,
};

use super::{
	macros::{self, Macro},
	scope::{ChildMode, ScopeLine, ScopeParenthesized, ScopedStream},
	NodeKind,
};

#[derive(Clone)]
pub struct Context<'a> {
	input: ScopedStream<'a>,
}

impl<'a> Context<'a> {
	pub fn new(input: Stream<'a>) -> Self {
		Context {
			input: ScopedStream::new(input),
		}
	}

	pub fn add_error(&mut self, error: Error<'a>) {
		self.input.add_error(error);
	}

	pub fn finish(self, program: Vec<NodeKind>) -> (Vec<NodeKind>, Vec<Error<'a>>) {
		(program, self.input.errors())
	}

	pub fn is_valid(&self) -> bool {
		!self.has_errors()
	}

	pub fn get_macro(&self, name: &str) -> Option<Box<dyn Macro>> {
		let out: Box<dyn Macro> = match name {
			"let" | "const" => Box::new(macros::Let),
			"print" => Box::new(macros::Print),
			"if" => Box::new(macros::If),
			"for" => Box::new(macros::For),
			_ => return None,
		};
		Some(out)
	}

	#[allow(unused)]
	pub fn scope_to_line(&mut self) {
		self.input.enter(ScopeLine::new(), ChildMode::Secondary);
	}

	pub fn scope_to_line_with_break(&mut self, split: &'static str) {
		self.input
			.enter(ScopeLine::new_with_break(split), ChildMode::Secondary);
	}

	pub fn scope_to_parenthesis(&mut self) {
		self.input
			.enter(ScopeParenthesized::new(), ChildMode::Override);
	}

	pub fn leave_scope(&mut self) {
		self.input.leave();
	}
}

impl<'a> LexStream<'a> for Context<'a> {
	fn copy(&self) -> Box<dyn LexStream<'a> + 'a> {
		Box::new(self.clone())
	}

	fn source(&self) -> &'a dyn Input {
		self.input.source()
	}

	fn next(&self) -> Lex<'a> {
		self.input.next()
	}

	fn advance(&mut self) {
		self.input.advance()
	}

	fn add_error(&mut self, error: Error<'a>) {
		self.input.add_error(error)
	}

	fn errors(&self) -> Vec<Error<'a>> {
		self.input.errors()
	}

	fn has_errors(&self) -> bool {
		self.input.has_errors()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::lexer::{self, Token};

	#[test]
	fn root_scope() {
		let input = "+ -";
		let mut context = Context::new(lexer::open(&input));

		assert_eq!(context.token(), Token::Symbol("+"));
		context.advance();

		assert_eq!(context.token(), Token::Symbol("-"));
		context.advance();

		assert_eq!(context.token(), Token::None);
	}

	#[test]
	fn line_scope() {
		let input = "+\n-\n*\n.";
		let mut context = Context::new(lexer::open(&input));

		// line scope should stop at the line break
		context.scope_to_line();
		assert_eq!(context.token(), Token::Symbol("+"));
		context.advance();
		assert_eq!(context.token(), Token::None);
		context.advance();
		assert_eq!(context.token(), Token::None);
		context.advance();

		// once the scope is back to root, the break can be read
		context.leave_scope();
		assert_eq!(context.token(), Token::Break);
		context.advance();
		assert_eq!(context.token(), Token::Symbol("-"));

		// next line scope
		context.scope_to_line();
		assert_eq!(context.token(), Token::Symbol("-"));
		context.advance();
		assert_eq!(context.token(), Token::None);

		// back to root
		context.leave_scope();
		assert_eq!(context.token(), Token::Break);
		context.advance();
		assert_eq!(context.token(), Token::Symbol("*"));
		context.advance();
		assert_eq!(context.token(), Token::Break);

		// create a line scope right at the end
		context.scope_to_line();
		assert_eq!(context.token(), Token::None);

		// and pop it
		context.leave_scope();
		assert_eq!(context.token(), Token::Break);
		context.advance();
		assert_eq!(context.token(), Token::Symbol("."));
		context.advance();
		assert_eq!(context.token(), Token::None);
	}

	#[test]
	fn line_scope_with_break() {
		let input = "+;-\n1; 2";
		let mut context = Context::new(lexer::open(&input));

		// line scope should stop at the line break
		context.scope_to_line_with_break(";");
		assert_eq!(context.token(), Token::Symbol("+"));
		context.advance();
		assert_eq!(context.token(), Token::None);

		// once the scope is back to root, the break can be read
		context.leave_scope();
		assert_eq!(context.token(), Token::Symbol(";"));
		context.advance();
		assert_eq!(context.token(), Token::Symbol("-"));

		// next line scope
		context.scope_to_line_with_break(";");
		assert_eq!(context.token(), Token::Symbol("-"));
		context.advance();
		assert_eq!(context.token(), Token::None);

		// back to root
		context.leave_scope();
		assert_eq!(context.token(), Token::Break);
		context.advance();
		assert_eq!(context.token(), Token::Integer(1));

		context.scope_to_line_with_break(";");
		assert_eq!(context.token(), Token::Integer(1));
		context.advance();
		assert_eq!(context.token(), Token::None);

		context.leave_scope();
		assert_eq!(context.token(), Token::Symbol(";"));
		context.advance();
		assert_eq!(context.token(), Token::Integer(2));
		context.advance();
		assert_eq!(context.token(), Token::None);
	}

	#[test]
	fn line_scope_parenthesis() {
		let input = "(1 2) 3";

		let mut context = Context::new(lexer::open(&input));
		assert_eq!(context.token(), Token::Symbol("("));

		context.scope_to_parenthesis();
		assert_eq!(context.token(), Token::Integer(1));
		context.advance();

		assert_eq!(context.token(), Token::Integer(2));
		context.advance();

		assert_eq!(context.token(), Token::None);
		context.advance();
		assert_eq!(context.token(), Token::None);

		context.leave_scope();
		assert_eq!(context.token(), Token::Symbol(")"));
		context.advance();
		assert_eq!(context.token(), Token::Integer(3));
		context.advance();
		assert_eq!(context.token(), Token::None);

		assert!(context.is_valid());
	}
}
