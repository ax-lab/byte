use std::{collections::VecDeque, error::Error, io::Write, sync::Arc};

use super::*;

pub const MAX_ERRORS: usize = 32;

/// List of errors.
#[derive(Clone, Default)]
pub struct Errors {
	list: Arc<VecDeque<Value>>,
}

impl Errors {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn from<T: IsValue>(error: T) -> Self {
		let mut errors = Self::new();
		errors.add(error);
		errors
	}

	pub fn from_list<T: IntoIterator<Item = V>, V: IsValue>(errors: T) -> Self {
		let list = errors.into_iter().map(|x| Value::from(x)).collect();
		Errors {
			list: Arc::new(list),
		}
	}

	pub fn empty(&self) -> bool {
		self.list.len() == 0
	}

	pub fn len(&self) -> usize {
		self.list.len()
	}

	pub fn append(&mut self, errors: &Errors) {
		let list = Arc::make_mut(&mut self.list);
		list.extend(errors.list.iter().cloned())
	}

	pub fn add<T: IsValue>(&mut self, error: T) {
		let list = Arc::make_mut(&mut self.list);
		list.push_back(Value::from(error));
	}

	pub fn iter(&self) -> ErrorIterator {
		ErrorIterator {
			next: 0,
			list: self.list.clone(),
		}
	}
}

//====================================================================================================================//
// Iterator
//====================================================================================================================//

pub struct ErrorIterator {
	next: usize,
	list: Arc<VecDeque<Value>>,
}

impl Iterator for ErrorIterator {
	type Item = Value;

	fn next(&mut self) -> Option<Self::Item> {
		if let Some(next) = self.list.get(self.next) {
			self.next += 1;
			Some(next.clone())
		} else {
			None
		}
	}
}

//====================================================================================================================//
// Traits
//====================================================================================================================//

impl Error for Errors {}

impl HasRepr for Errors {
	fn output_repr(&self, output: &mut Repr<'_>) -> std::io::Result<()> {
		if self.len() == 0 {
			if output.is_debug() {
				write!(output, "<NoErrors>")
			} else {
				write!(output, "")
			}
		} else {
			if output.is_debug() {
				write!(output, "<Errors")?;
			} else {
				write!(output, "Errors:\n")?;
			}

			{
				let mut output = output.indented();
				for (n, it) in self.iter().enumerate() {
					write!(output, "\n[{}]", n + 1)?;
					let mut output = if let Some(span) = it.get_span() {
						write!(output, " at ")?;
						span.output_repr(&mut output.display().full())?;
						write!(output, "\n")?;
						output.indented()
					} else {
						write!(output, " ")?;
						output.clone()
					};
					it.output_repr(&mut output)?;
				}
			}

			if output.is_debug() {
				write!(output, "\n>")?;
			} else {
				write!(output, "\n")?;
			}
			Ok(())
		}
	}
}

fmt_from_repr!(Errors);

impl std::ops::Index<usize> for Errors {
	type Output = Value;

	fn index(&self, index: usize) -> &Self::Output {
		&self.list[index]
	}
}

//====================================================================================================================//
// Tests
//====================================================================================================================//

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_output() {
		let input = Input::new("input.txt", "12\n\n\t\n");
		let mut input = input.cursor();

		let p11 = input.clone();
		input.read();
		input.read();
		let p13 = input.clone();
		input.read();
		let p21 = input.clone();
		input.read();
		input.read();
		let p35 = input.clone();
		input.read();
		let p41 = input.clone();

		let mut errors = Errors::default();
		errors.add("some error 1");
		errors.add("some error 2".to_string());
		errors.add("error A".at(Span::from(&p11, &p13)));
		errors.add("error B".at(Span::from(&p21, &p21)));
		errors.add("error C\n    with some detail".at(Span::from(&p35, &p41)));

		let expected = vec![
			"Errors:",
			"    ",
			"    [1] some error 1",
			"    [2] some error 2",
			"    [3] at input.txt:1:1…1:3",
			"        error A",
			"    [4] at input.txt:2:1",
			"        error B",
			"    [5] at input.txt:3:5…4:1",
			"        error C",
			"            with some detail",
			"",
		];
		let expected = expected.join("\n");
		let actual = errors.to_string();
		assert_eq!(actual, expected);
	}
}
