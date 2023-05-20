pub mod declare;
pub mod node;
pub mod segments;

pub use node::*;
pub use segments::*;

use std::io::Write;

use crate::core::*;
use crate::lexer::*;

pub trait IsNode: IsValue + WithEquality {}

pub trait SyntaxMacro {
	fn parse(&self, stream: &mut NodeStream, errors: &mut Errors) -> Option<Node>;

	fn valid_symbols(&self) -> Option<&[&'static str]> {
		None
	}
}
