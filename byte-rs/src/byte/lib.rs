pub mod code;
pub mod context;
pub mod eval;
pub mod lexer;
pub mod names;
pub mod nodes;
pub mod precedence;
pub mod util;

pub use context::*;
pub use lexer::*;
pub use names::*;
pub use nodes::*;
pub use precedence::*;
pub use util::*;

pub type Result<T> = std::result::Result<T, Errors>;

use std::{
	fmt::{Debug, Display, Formatter, Write},
	sync::Arc,
};

pub fn new() -> Context {
	let mut scanner = Scanner::with_common_symbols();
	scanner.add_matcher(CommentMatcher);
	scanner.add_matcher(LiteralMatcher);
	scanner.add_matcher(IntegerMatcher);
	Context::new_root(scanner)
}

pub fn eval(input: &Input) -> Result<Value> {
	let context = new();
	let span = input.start().span();
	let text = Node::from(RawText(input.clone()), Some(span));
	let (context, nodes) = context.resolve_all(NodeList::single(text))?;
	println!("{nodes:?}");

	let _ = (context, nodes);
	todo!();
	// let expr = compile(context, nodes)?;
	// let runtime = Runtime::new();
	// expr.eval(&mut runtime)
}

pub fn eval_string<T: AsRef<str>>(input: T) -> Result<Value> {
	let data = input.as_ref().as_bytes();
	let input = Input::new("eval_string", data.to_vec());
	eval(&input)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn empty() -> Result<()> {
		assert_eq!(eval_string("")?, Value::from(()));
		Ok(())
	}

	#[test]
	fn hello() -> Result<()> {
		assert_eq!(eval_string("'hello world'")?, Value::from("hello world".to_string()));
		Ok(())
	}

	#[test]
	fn the_answer() -> Result<()> {
		assert_eq!(eval_string("42")?, Value::from(42));
		Ok(())
	}
}
