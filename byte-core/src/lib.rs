pub mod lexer;
pub mod tokens;

pub fn name() -> &'static str {
	"Byte Language"
}

pub fn version() -> &'static str {
	"0.1.0"
}

pub fn exec<T: AsRef<str>>(input: T) -> ExecResult {
	let mut stdout = String::new();
	for line in input.as_ref().lines() {
		let line = line.strip_prefix("print").unwrap_or(line).trim();
		let line = line.trim_matches('\'');
		if line.len() > 0 {
			if stdout.len() > 0 {
				stdout.push('\n');
			}
			stdout.push_str(line);
		}
	}
	ExecResult { stdout }
}

pub struct ExecResult {
	stdout: String,
}

impl ExecResult {
	pub fn success(&self) -> bool {
		true
	}

	pub fn stdout(&self) -> &str {
		&self.stdout
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn it_works() {
		assert!(name().contains("Byte"));
		assert!(version() != "");
	}
}
