use crate::core::error::*;
use crate::core::input::*;
use crate::lexer::*;

use super::node::NodeKind;

use super::{
	macros::{self, Macro},
	scope::{ChildMode, Scope, ScopeLine, ScopeParenthesized, ScopedStream},
};

#[derive(Clone)]
pub struct Context {
	input: ScopedStream,
}

impl Context {
	pub fn new(input: Lexer) -> Self {
		Context {
			input: ScopedStream::new(input),
		}
	}

	pub fn finish(self, program: Vec<NodeKind>) -> (Vec<NodeKind>, Vec<Error>) {
		(program, self.input.list_errors())
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

	pub fn enter_scope(&mut self, scope: Box<dyn Scope>) {
		self.input.enter(scope, ChildMode::Secondary);
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

impl Stream for Context {
	fn pos(&self) -> Cursor {
		self.input.pos()
	}

	fn copy(&self) -> Box<dyn Stream> {
		self.input.copy()
	}

	fn next(&self) -> TokenAt {
		self.input.next()
	}

	fn read(&mut self) -> TokenAt {
		self.input.read()
	}

	fn errors(&self) -> ErrorList {
		self.input.errors()
	}

	fn add_error(&mut self, error: Error) {
		self.input.add_error(error)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	use crate::lang::Integer;

	fn open(str: &'static str) -> Lexer {
		let input = Input::open_str(str, str);
		crate::lexer::open(input)
	}

	#[test]
	fn root_scope() {
		let input = "+ -";
		let mut context = Context::new(open(input));

		assert_eq!(context.token(), Token::Symbol("+"));
		context.advance();

		assert_eq!(context.token(), Token::Symbol("-"));
		context.advance();

		assert_eq!(context.token(), Token::None);
	}

	#[test]
	fn line_scope() {
		let input = "+\n-\n*\n.";
		let mut context = Context::new(open(input));

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
		let mut context = Context::new(open(input));

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
		assert_eq!(context.token(), Integer::token(1));

		context.scope_to_line_with_break(";");
		assert_eq!(context.token(), Integer::token(1));
		context.advance();
		assert_eq!(context.token(), Token::None);

		context.leave_scope();
		assert_eq!(context.token(), Token::Symbol(";"));
		context.advance();
		assert_eq!(context.token(), Integer::token(2));
		context.advance();
		assert_eq!(context.token(), Token::None);
	}

	#[test]
	fn line_scope_parenthesis() {
		let input = "(1 2) 3";

		let mut context = Context::new(open(input));
		assert_eq!(context.token(), Token::Symbol("("));

		context.scope_to_parenthesis();
		assert_eq!(context.token(), Integer::token(1));
		context.advance();

		assert_eq!(context.token(), Integer::token(2));
		context.advance();

		assert_eq!(context.token(), Token::None);
		context.advance();
		assert_eq!(context.token(), Token::None);

		context.leave_scope();
		assert_eq!(context.token(), Token::Symbol(")"));
		context.advance();
		assert_eq!(context.token(), Integer::token(3));
		context.advance();
		assert_eq!(context.token(), Token::None);

		assert!(context.is_valid());
	}
}
