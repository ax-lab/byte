use std::collections::HashMap;

mod value;
pub use value::*;

pub struct Runtime {
	vars: HashMap<String, Value>,
}

impl Runtime {
	pub fn new() -> Self {
		Runtime {
			vars: Default::default(),
		}
	}

	pub fn get(&self, name: &str) -> Value {
		self.vars
			.get(name)
			.cloned()
			.unwrap_or_else(|| panic!("variable {name} not defined"))
	}

	#[allow(unused)]
	pub fn set(&mut self, name: &str, value: Value) -> Option<Value> {
		if let Some(entry) = self.vars.get_mut(name) {
			let previous = std::mem::replace(entry, value);
			Some(previous.as_value())
		} else {
			self.vars.insert(name.into(), value.to_ref());
			None
		}
	}
}
