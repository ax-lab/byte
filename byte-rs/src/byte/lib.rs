pub mod context;
pub use context::*;

pub mod code;
pub mod compiler;
pub mod errors;
pub mod nodes;
pub mod offset;
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
pub use offset::*;
pub use program::*;
pub use scanner::*;
pub use scope::*;
pub use span::*;
pub use util::*;
pub use values::*;

/*
	Laundry list (TODO)
	===================

	## Clean up

	- [ ] have proper scoping
	- [ ] tidy up the code module

	## Major systems

	- [ ] import / export
	- [ ] native compilation (C / LLVM)
	- [ ] JS transpilation

	## Language features

	- [ ] types
	- [ ] pattern matching
	- [ ] macros
	- [ ] lifetime and ownership (a.k.a. borrows)
	- [ ] lexical / syntax extensions from code

	## General code improvements

	- [ ] stricter type-checking in codegen (e.g. strongly-typed generic code nodes?)
	- [ ] generalize framework for type conversion
	- [ ] isomorphic code gen and eval
	- [ ] improve segment handling and resolving (e.g. if-else problem -- defer to code gen?)
		- [ ] e.g., add a formal code analysis step'

	Next Language Milestones
	========================

	- Import / Modules
	- Functions
	- Type declarations
	- Pattern matching
	- Borrows / Timeline
	- JS transpilation
	- Native compilation

*/

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

const DEBUG_PROCESSING: bool = false;
const DEBUG_PROCESSING_DETAIL: bool = false;

pub type Result<T> = std::result::Result<T, Errors>;

pub fn id() -> Id {
	Context::id()
}

// TODO: remove `at(span)`
pub fn at(span: Span) -> Span {
	span
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

		let mut node = program.load_string("hello", "print 'hello world!!!'")?;
		program.run_node(&mut node)?;

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

	#[test]
	#[ignore = "build the language so this is possible"]
	fn muncher() -> Result<()> {
		/*
			The idea here is to have the concept of a muncher: a "normal" type
			member (i.e. `point.new`) that is able to parse raw nodes and
			result in a value or expression.

			The first challenge is that node resolution needs to be dynamic
			and powerful enough to enable this sort of resolution during
			parsing.

			The type system also needs to support this concept. This requires
			it to be tightly integrated with the nodes.

			Parsing is also a challenge, as the `point.new` needs to be parsed
			early in the expression parsing so that it can have access to the
			raw nodes.

			For this to happen:

			- the `point` binding needs to happen early enough for it to be
			  bound to the type by the time the expression is being parsed.
			- the expression parser needs to be able to resolve the member
			  operator mid-parsing to have access to `point.new`
			- there must be an interface that allows the expression parser
			  to then use the muncher to parse as many tokens as it wants

			A similar concept would also apply at a higher level for syntax
			macros.

			This calls for an extension of the node evaluators, where a node
			itself provides contextual evaluation operators. This would get
			sorted by precedence and then applied at the appropriate moment.
		*/
		let compiler = Compiler::new();
		let source = vec![
			"let a = point.new (0, 0)",
			"let b = point.new x=1, y=2",
			"let c = point.new 1, 2, 3",
			"let d = point.new x=10, y=20, z=30",
			"print a",
			"print b",
			"print c",
			"print d",
			"d",
		];
		let source = source.join("\n");

		let output: Arc<RwLock<Vec<u8>>> = Default::default();
		let mut program = compiler.new_program();
		program.configure_runtime(|rt| {
			let output = RuntimeOutput::Memory(output.clone());
			rt.redirect_stdout(output);
		});

		let result = compiler.eval_string(source)?;
		assert_eq!(result, Point3D::new(10.0, 20.0, 30.0));

		let output = output.read().unwrap().clone();
		let output = String::from_utf8(output)?;
		assert_eq!(output, "P(0, 0) P(1, 2) P(1, 2, 3) P(10, 20, 30)\n");

		Ok(())
	}

	struct Point2D {
		x: f64,
		y: f64,
	}

	struct Point3D {
		x: f64,
		y: f64,
		z: f64,
	}

	impl Point2D {}

	impl Point3D {
		pub fn new(x: f64, y: f64, z: f64) -> Value {
			let _ = (x, y, z);
			todo!()
		}
	}

	impl Display for Point2D {
		fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
			write!(f, "P({}, {})", self.x, self.y)
		}
	}

	impl Display for Point3D {
		fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
			write!(f, "P({}, {}, {})", self.x, self.y, self.z)
		}
	}
}
