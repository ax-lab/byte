pub mod core;

pub mod compiler;
pub mod lang;
pub mod lexer;
pub mod nodes;
pub mod runtime;

pub use crate::core::*;

pub use compiler::*;
pub use nodes::*;
pub use runtime::Runtime;

type Result<T> = std::result::Result<T, Errors>;

pub fn new() -> Context {
	Context::new_with_defaults()
}

pub fn run(input: Input, _rt: &mut Runtime) -> Result<Var> {
	let _ = compile(input)?;
	todo!()
}

pub fn compile(input: Input) -> Result<runtime::Code> {
	let mut context = new();
	context.load_input(input);
	context.wait_resolve();

	let errors = context.errors();
	if errors.len() > 0 {
		return Err(errors);
	}

	todo!()
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn answer_to_everything() {
		let mut rt = Runtime::default();
		let result = run(Input::from("42"), &mut rt).unwrap();
		assert_eq!(result.value(), Value::from(42))
	}

	#[test]
	fn hello_world() {
		let mut rt = Runtime::default();
		let mut output = String::new();
		rt.redirect_output(&mut output);

		let _ = run(Input::from("print 'hello world!!!'"), &mut rt).unwrap();
		assert_eq!(output, "hello world!!!\n");
	}
}
