use std::sync::{Arc, RwLock};

pub struct Runtime {
	output: Option<Box<dyn std::io::Write>>,
}

impl Default for Runtime {
	fn default() -> Self {
		Self {
			output: Some(Box::new(std::io::stdout())),
		}
	}
}

impl Runtime {
	pub fn redirect_output(&mut self, output: Box<dyn std::io::Write>) {
		self.output = Some(output);
	}
}

#[derive(Default, Clone)]
pub struct StringOutput {
	buffer: Arc<RwLock<Vec<u8>>>,
}

impl StringOutput {
	pub fn read(&self) -> String {
		let buffer = self.buffer.read().unwrap();
		String::from_utf8_lossy(&buffer).to_string()
	}

	pub fn writer(&mut self) -> StringOutputWriter {
		StringOutputWriter {
			buffer: self.buffer.clone(),
		}
	}
}

pub struct StringOutputWriter {
	buffer: Arc<RwLock<Vec<u8>>>,
}

impl std::io::Write for StringOutputWriter {
	fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
		let mut buffer = self.buffer.write().unwrap();
		buffer.extend(buf.iter());
		Ok(buf.len())
	}

	fn flush(&mut self) -> std::io::Result<()> {
		Ok(())
	}
}
