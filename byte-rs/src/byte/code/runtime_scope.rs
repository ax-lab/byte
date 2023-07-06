use std::collections::HashMap;

use super::*;

pub struct RuntimeScope {
	values: HashMap<(Symbol, Option<usize>), Value>,
	stdout: RuntimeOutput,
	stderr: RuntimeOutput,
}

impl RuntimeScope {
	pub fn new() -> Self {
		Default::default()
	}

	pub fn set(&mut self, name: Symbol, index: Option<usize>, value: Value) -> Option<Value> {
		self.values.insert((name, index), value)
	}

	pub fn get(&self, name: &Symbol, index: Option<usize>) -> Option<&Value> {
		self.values.get(&(name.clone(), index))
	}

	pub fn stdout(&self) -> RuntimeOutput {
		self.stdout.clone()
	}

	pub fn stderr(&self) -> RuntimeOutput {
		self.stderr.clone()
	}

	pub fn redirect_stdout(&mut self, output: RuntimeOutput) -> RuntimeOutput {
		std::mem::replace(&mut self.stdout, output)
	}

	pub fn redirect_stderr(&mut self, output: RuntimeOutput) -> RuntimeOutput {
		std::mem::replace(&mut self.stderr, output)
	}
}

impl Default for RuntimeScope {
	fn default() -> Self {
		Self {
			values: Default::default(),
			stdout: RuntimeOutput::StdOut,
			stderr: RuntimeOutput::StdErr,
		}
	}
}

#[derive(Clone)]
pub enum RuntimeOutput {
	None,
	StdOut,
	StdErr,
	Memory(Arc<RwLock<Vec<u8>>>),
	Writer(Arc<RwLock<Box<dyn std::io::Write>>>),
}

impl RuntimeOutput {
	fn exec<T, P: FnOnce(&mut dyn std::io::Write) -> std::io::Result<T>>(&mut self, action: P) -> std::io::Result<T> {
		match self {
			RuntimeOutput::StdOut => {
				let mut output = std::io::stdout().lock();
				(action)(&mut output)
			}
			RuntimeOutput::StdErr => {
				let mut output = std::io::stderr().lock();
				(action)(&mut output)
			}
			RuntimeOutput::None => {
				let mut output = VoidWriter;
				(action)(&mut output)
			}
			RuntimeOutput::Memory(output) => {
				let mut output = match output.write() {
					Ok(output) => output,
					Err(err) => err.into_inner(),
				};
				let output = &mut *output;
				(action)(output)
			}
			RuntimeOutput::Writer(output) => {
				let mut output = match output.write() {
					Ok(output) => output,
					Err(err) => err.into_inner(),
				};
				(action)(&mut *output)
			}
		}
	}
}

impl std::io::Write for RuntimeOutput {
	fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
		self.exec(|output| output.write(buf))
	}

	fn flush(&mut self) -> std::io::Result<()> {
		self.exec(|output| output.flush())
	}
}

impl std::fmt::Write for RuntimeOutput {
	fn write_str(&mut self, s: &str) -> std::fmt::Result {
		use std::io::Write;
		match self.write(s.as_bytes()) {
			Ok(..) => Ok(()),
			Err(..) => Err(std::fmt::Error),
		}
	}
}

struct VoidWriter;

impl std::io::Write for VoidWriter {
	fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
		Ok(buf.len())
	}

	fn flush(&mut self) -> std::io::Result<()> {
		Ok(())
	}
}
