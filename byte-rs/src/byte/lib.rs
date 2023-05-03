pub mod core;
pub mod lexer;
pub mod nodes;
pub mod runtime;

pub use crate::core::*;
pub use crate::nodes::*;
pub use crate::runtime::Runtime;

type Result<T> = std::result::Result<T, Errors>;

pub fn run(_input: &Input, _rt: &mut Runtime) -> Result<Var> {
	todo!()
}

pub fn compile(_input: &Input, _rt: &mut Runtime) -> Result<runtime::Code> {
	todo!()
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn answer_to_everything() {
		let mut rt = Runtime::default();
		let result = run(&Input::from("42"), &mut rt).unwrap();
		assert_eq!(result.value(), Value::from(42))
	}

	#[test]
	fn hello_world() {
		let mut rt = Runtime::default();
		let mut output = String::new();
		rt.redirect_output(&mut output);

		let _ = run(&Input::from("print 'hello world!!!'"), &mut rt).unwrap();
		assert_eq!(output, "hello world!!!\n");
	}
}
