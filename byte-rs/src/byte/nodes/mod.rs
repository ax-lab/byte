pub mod declare;
pub mod node;
pub mod segments;

pub use node::*;
pub use segments::*;

use std::io::Write;

use crate::build::*;
use crate::code::*;
use crate::core::*;
use crate::lexer::*;

pub trait IsNode: IsValue + WithEquality {
	fn resolve(&self, context: &Context) -> ResolveResult {
		let _ = context;
		ResolveResult::Done
	}

	fn finalize(&self, context: &Context) {
		let _ = context;
	}
}

pub trait IsCompilable {
	fn compile(&self, context: &Context) -> Option<Expr>;
}

pub trait SyntaxMacro {
	fn parse(&self, stream: &mut NodeStream, errors: &mut Errors) -> Option<Node>;

	fn valid_symbols(&self) -> Option<&[&'static str]> {
		None
	}
}
