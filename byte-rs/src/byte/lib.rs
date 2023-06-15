pub mod code;
pub mod compiler;
pub mod context;
pub mod lexer;
pub mod nodes;
pub mod precedence;
pub mod util;

pub use code::*;
pub use compiler::*;
pub use context::*;
pub use lexer::*;
pub use nodes::*;
pub use precedence::*;
pub use util::*;

const MAX_ERRORS: usize = 16;

pub type Result<T> = std::result::Result<T, Errors>;

use std::{
	collections::HashSet,
	fmt::{Debug, Display, Formatter, Write},
	sync::{Arc, RwLock, Weak},
};

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn empty() -> Result<()> {
		let compiler = Compiler::new();
		assert_eq!(compiler.eval_string("")?, Value::from(()));
		Ok(())
	}

	#[test]
	fn hello() -> Result<()> {
		let compiler = Compiler::new();
		assert_eq!(
			compiler.eval_string("'hello world'")?,
			Value::from("hello world".to_string())
		);
		Ok(())
	}

	#[test]
	fn the_answer() -> Result<()> {
		let compiler = Compiler::new();
		assert_eq!(compiler.eval_string("42")?, Value::from(42i64));
		Ok(())
	}
}
