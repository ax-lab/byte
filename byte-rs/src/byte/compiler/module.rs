use std::path::Path;

use super::*;

/// Represents an isolated module of code.
#[derive(Clone)]
pub struct Module {
	input: Input,
}

impl Module {
	pub fn from_path<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
		let input = Input::open(path)?;
		Ok(Self { input })
	}

	pub fn from_input(input: Input) -> Self {
		Self { input }
	}

	pub fn input(&self) -> &Input {
		&self.input
	}
}
