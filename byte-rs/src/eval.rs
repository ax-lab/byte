#![allow(unused)]

use crate::core::input::*;
use crate::core::*;
use crate::nodes;
use crate::old::stream::Stream;
use crate::parser;
use crate::vm::*;

#[allow(unused)]
pub struct Output {
	result: Var,
	output: String,
}

#[allow(unused)]
impl Output {
	pub fn val(&self) -> Value {
		self.result.val()
	}

	pub fn output(&self) -> &str {
		&self.output
	}
}

#[allow(unused)]
pub fn eval<T: AsRef<str>>(rt: &mut Runtime, input: T) -> Result<Output, ErrorList> {
	// let input = Input::open_str("eval", input);
	// let mut lexer = parser::open(input);

	// let mut block = Vec::new();
	// while let Some(next) = parser::parse_next(&mut lexer) {
	// 	if lexer.has_errors() || next.has_errors() {
	// 		let mut errors = lexer.errors();
	// 		errors.append(next.errors());
	// 		return Err(errors);
	// 	}
	// 	block.push(next);
	// }

	// let block = nodes::Block::new(block);
	// block.resolve(rt);

	// let code = block.compile()?;

	// let mut output = String::new();
	// let mut rt = rt.redirect_output(&mut output);
	// let result = rt.execute(code);

	// Ok(Output { result, output })
	todo!()
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn answer_to_everything() {
		let mut rt = Runtime::new();
		let result = eval(&mut rt, "42").unwrap();
		assert_eq!(result.val(), Value::any_int(42))
	}

	#[test]
	fn hello_world() {
		let mut rt = Runtime::new();
		let result = eval(&mut rt, "print 'hello world!!!'").unwrap();
		assert_eq!(result.output(), "hello world!!!\n");
	}
}
