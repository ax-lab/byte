use super::*;

#[derive(Default)]
pub struct Scope {}

impl Scope {
	pub fn new() -> Self {
		Default::default()
	}

	pub fn get(&self, name: &str) -> Value {
		let _ = name;
		todo!()
	}

	pub fn set(&self, name: &str, value: Value) {
		let _ = (name, value);
		todo!()
	}
}
