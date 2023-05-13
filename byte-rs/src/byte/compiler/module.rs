use super::*;

/// Represents an isolated module of code.
#[derive(Clone)]
pub struct Module {
	input: Input,
}

impl Module {
	pub fn from_input(input: Input) -> Self {
		Self { input }
	}

	pub fn input(&self) -> &Input {
		&self.input
	}
}
