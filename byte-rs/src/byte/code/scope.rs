use std::collections::HashMap;

use super::*;

#[derive(Default)]
pub struct RuntimeScope {
	names: HashMap<Name, (usize, Type)>,
	values: Vec<Option<Value>>,
}

impl RuntimeScope {
	pub fn new() -> Self {
		Default::default()
	}

	pub fn declare(&mut self, name: Name, kind: Type) -> Result<()> {
		let names = &mut self.names;
		let values = &mut self.values;
		if let Some((.., kind)) = names.get(&name) {
			Err(Errors::from(format!(
				"variable `{name}` already declared in the scope (type: {kind:?})"
			)))
		} else {
			let index = values.len();
			names.insert(name, (index, kind));
			values.push(None);
			Ok(())
		}
	}

	pub fn get(&self, name: &Name) -> Result<&Value> {
		if let Some((index, kind)) = self.names.get(name) {
			if let Some(value) = self.values.get(*index).and_then(|x| x.as_ref()) {
				Ok(value)
			} else {
				Err(Errors::from(format!(
					"cannot get uninitialized variable `{name}` (type: {kind:?})"
				)))
			}
		} else {
			Err(Errors::from(format!("cannot get undeclared variable `{name}`")))
		}
	}

	pub fn set(&mut self, name: &Name, value: Value) -> Result<()> {
		if let Some((index, kind)) = self.names.get(name) {
			kind.validate_value(&value)?;
			self.values[*index] = Some(value);
			Ok(())
		} else {
			Err(Errors::from(format!("cannot set undeclared variable `{name}`")))
		}
	}
}
