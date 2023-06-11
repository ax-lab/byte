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

const MAX_ERRORS: usize = 16;

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

	let mut code = Vec::new();
	let mut errors = Errors::new();
	for it in nodes.iter() {
		if let Some(node) = it.as_compilable() {
			if let Some(item) = node.compile(it, &context, &mut errors) {
				code.push(item);
			}
		} else {
			errors.add_at(
				format!("resulting node is not compilable -- {it:?}"),
				it.span().cloned(),
			);
		}

		if errors.len() > MAX_ERRORS {
			break;
		}
	}

	if errors.len() > 0 {
		return Err(errors);
	}

	let mut value = Value::from(());
	let mut scope = eval::Scope::new();
	for it in code {
		value = it.eval(&mut scope)?;
	}

	Ok(value)
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
		assert_eq!(eval_string("42")?, Value::from(42i64));
		Ok(())
	}
}
