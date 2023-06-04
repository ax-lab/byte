use super::*;

#[derive(Default)]
pub struct Scope {}

impl Scope {
	pub fn new() -> Self {
		Default::default()
	}

	pub fn get(&self, name: &Name) -> Value {
		let _ = name;
		todo!()
	}

	pub fn set(&self, name: &Name, value: Value) {
		let _ = (name, value);
		todo!()
	}
}
