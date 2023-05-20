pub mod core;

pub mod build;
pub mod code;
pub mod exec;
pub mod lang;
pub mod lexer;
pub mod nodes;
pub mod runtime;

pub use crate::core::*;

pub use build::*;
pub use code::*;
pub use nodes::*;
pub use runtime::Runtime;

type Result<T> = std::result::Result<T, Errors>;

pub fn new() -> Context {
	Context::new_with_defaults()
}

pub fn run(input: Input, rt: &mut Runtime) -> Result<Value> {
	let code = compile(input)?;
	code.execute(rt).map_err(|err| {
		let mut errors = Errors::new();
		errors.add(err.to_string());
		errors
	})
}

pub fn compile(input: Input) -> Result<Code> {
	let mut context = new();

	let module = context.load_input(input);
	context.resolve_all();

	let errors = context.errors();
	if errors.len() > 0 {
		return Err(errors);
	}

	Ok(module.code())
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn answer_to_everything() {
		let mut rt = Runtime::default();
		let result = run(Input::from("42"), &mut rt).unwrap();
		assert_eq!(result, Value::from(42))
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
