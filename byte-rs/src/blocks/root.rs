use crate::lexer::*;

use super::*;

#[derive(Clone)]
pub struct Root {}

impl IsBlock for Root {
	type Value = Vec<Statement>;

	fn parse<T: Scope>(mut scope: T) -> (T, Self::Value) {
		let mut out = Vec::new();
		while scope.peek().is_some() {
			scope = {
				let scope = StatementScope::new(scope);
				let (scope, next) = Statement::parse(scope);
				out.push(next);
				scope.into_inner()
			};
		}
		(scope, out)
	}
}

#[derive(Clone)]
pub struct RootScope {
	stream: TokenStream,
	indent: Indent,
}

impl RootScope {
	pub fn new(stream: TokenStream) -> Self {
		Self {
			stream,
			indent: Indent::default(),
		}
	}
}

impl Scope for RootScope {
	fn read(&mut self) -> TokenAt {
		let stream = &mut self.stream;
		stream.skip();

		let start = stream.pos();
		let empty = start.col() == start.indent();
		loop {
			stream.skip();
			let start = stream.pos().clone();
			let errors = stream.errors_mut();
			let next = if let Some(next) = self.indent.check_indent(&start, errors) {
				next
			} else {
				stream.read()
			};
			let token = next.token();
			if token.is::<Comment>() || (empty && token == Token::Break) {
				continue;
			}
			break next;
		}
	}
}
