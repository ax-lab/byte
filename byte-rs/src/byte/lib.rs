pub mod context;
pub use context::*;

pub mod code;
pub mod compiler;
pub mod errors;
pub mod nodes;
pub mod program;
pub mod scanner;
pub mod scope;
pub mod span;
pub mod util;
pub mod values;

pub use code::*;
pub use compiler::*;
pub use errors::*;
pub use nodes::*;
pub use program::*;
pub use scanner::*;
pub use scope::*;
pub use span::*;
pub use util::*;
pub use values::*;

/// Default initial tab-width for the compiler.
///
/// This can be overridden at the [`Context`] or [`Source`] level.
///
/// The tab-width is used to compute column and indentation levels for tabs
/// in the source code. This is mostly visible when reporting a location.
///
/// As such, changing the tab-width has only "cosmetic" effects in compiler
/// messages and reported locations.
///
/// Changing the tab-width DOES change the relationship between space and tabs
/// in indentation, which would have semantic implications, except that
/// inconsistent use of tabs and spaces in the indentation of continuous lines
/// is forbidden.
const DEFAULT_TAB_WIDTH: usize = 4;

// TODO: create a "CompilerInfo" struct that can be apply to any value, node, or error, containing compiler source information.

const MAX_ERRORS: usize = 10;

const DUMP_CODE: bool = false;

pub type Result<T> = std::result::Result<T, Errors>;

pub fn id() -> Id {
	Context::id()
}

pub fn at(span: Span) -> Id {
	Context::id().at(span)
}

use std::{
	collections::HashMap,
	collections::HashSet,
	collections::VecDeque,
	fmt::{Debug, Display, Formatter, Write},
	hash::Hash,
	ops::{Deref, Range, RangeBounds},
	path::{Path, PathBuf},
	sync::{Arc, RwLock, Weak},
};

mod macros {
	#[macro_export]
	macro_rules! err {
		($($t:tt)*) => {{
			Err(Errors::from(format!($($t)*), Span::default()))
		}};
	}
}

pub use macros::*;

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
	fn hello_world() -> Result<()> {
		let compiler = Compiler::new();
		let mut program = compiler.new_program();

		let output: Arc<RwLock<Vec<u8>>> = Default::default();
		program.configure_runtime(|rt| {
			let output = RuntimeOutput::Memory(output.clone());
			rt.redirect_stdout(output);
		});

		let nodes = program.load_string("hello", "print 'hello world!!!'")?;
		program.run_nodes(&nodes)?;

		let output = output.read().unwrap().clone();
		let output = String::from_utf8(output)?;
		assert_eq!(output, "hello world!!!\n");

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
		assert_eq!(compiler.eval_string("42")?, Value::from(int(42)));
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
		assert_eq!(compiler.eval_string(source)?, Value::from(int(42)));
		Ok(())
	}
}

/*
	Operator rules
	==============

	1) All operators are tied to particular nodes in the list. If the given
	   node is not present, then the operator will not be applied.

	2) Operators will remove all instances of their respective nodes from the
	   list.

	3) Operators may result in a new list containing their respective nodes,
	   but should only process those recursively if the precedence of all
	   other operators can be ensured.

*/
