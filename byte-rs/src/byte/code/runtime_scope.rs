use std::collections::HashMap;

use super::*;

#[derive(Default)]
pub struct RuntimeScope {
	values: HashMap<(Name, Option<usize>), Value>,
}

impl RuntimeScope {
	pub fn new() -> Self {
		Default::default()
	}

	pub fn set(&mut self, name: Name, index: Option<usize>, value: Value) -> Option<Value> {
		self.values.insert((name, index), value)
	}

	pub fn get(&self, name: &Name, index: Option<usize>) -> Option<&Value> {
		self.values.get(&(name.clone(), index))
	}
}
