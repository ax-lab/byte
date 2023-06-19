pub mod code;
pub mod compiler;
pub mod lexer;
pub mod module;
pub mod nodes;
pub mod operators;
pub mod precedence;
pub mod resolve;
pub mod util;

pub use code::*;
pub use compiler::*;
pub use lexer::*;
pub use module::*;
pub use nodes::*;
pub use operators::*;
pub use precedence::*;
pub use resolve::*;
pub use util::*;

pub const MAX_ERRORS: usize = 16;

pub type Result<T> = std::result::Result<T, Errors>;

use std::{
	collections::HashMap,
	collections::HashSet,
	fmt::{Debug, Display, Formatter, Write},
	hash::Hash,
	ops::{Deref, RangeBounds},
	path::{Path, PathBuf},
	sync::{Arc, RwLock, Weak},
};

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn eval_empty() -> Result<()> {
		let compiler = Compiler::new();
		assert_eq!(compiler.eval_string("")?, Value::from(()));
		Ok(())
	}

	#[test]
	fn eval_hello() -> Result<()> {
		let compiler = Compiler::new();
		assert_eq!(
			compiler.eval_string("'hello world'")?,
			Value::from("hello world".to_string())
		);
		Ok(())
	}

	#[test]
	fn eval_the_answer() -> Result<()> {
		let compiler = Compiler::new();
		assert_eq!(compiler.eval_string("42")?, Value::from(42i64));
		Ok(())
	}

	#[test]
	fn eval_variable() -> Result<()> {
		let compiler = Compiler::new();
		let source = vec![
			"let a = 2",
			"let b = 5",
			"let c = b + b",
			"let result = c + c + c + b + b + a",
			"result",
		];
		let source = source.join("\n");
		assert_eq!(compiler.eval_string(source)?, Value::from(42i64));
		Ok(())
	}
}
