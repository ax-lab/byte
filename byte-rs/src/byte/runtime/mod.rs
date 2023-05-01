pub struct Runtime {}

impl Default for Runtime {
	fn default() -> Self {
		Self {}
	}
}

impl Runtime {
	pub fn redirect_output(&mut self, _output: &mut String) {
		todo!()
	}
}

pub struct Code {}
