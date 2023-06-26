use std::{collections::VecDeque, error::Error, sync::Arc};

use super::*;

pub const MAX_ERRORS: usize = 32;

/// List of errors.
#[derive(Clone, Default)]
pub struct Errors {
	list: Arc<VecDeque<(Value, Option<Span>)>>,
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

	pub fn empty(&self) -> bool {
		self.list.len() == 0
	}

	pub fn len(&self) -> usize {
		self.list.len()
	}

	pub fn append(&mut self, errors: &Errors) {
		if errors.len() > 0 {
			let list = Arc::make_mut(&mut self.list);
			list.extend(errors.list.iter().cloned())
		}
	}

	pub fn add<T: IsValue>(&mut self, error: T) {
		let list = Arc::make_mut(&mut self.list);
		list.push_back((Value::from(error), None));
	}

	pub fn add_at<T: IsValue>(&mut self, error: T, span: Span) {
		// TODO: implement span location
		let list = Arc::make_mut(&mut self.list);
		list.push_back((Value::from(error), Some(span)));
	}

	pub fn iter(&self) -> ErrorIterator {
		ErrorIterator {
			next: 0,
			list: self.list.clone(),
		}
	}
}

//====================================================================================================================//
// ErrorIterator
//====================================================================================================================//

pub struct ErrorData {
	data: Value,
	span: Option<Span>,
}

pub struct ErrorIterator {
	next: usize,
	list: Arc<VecDeque<(Value, Option<Span>)>>,
}

impl Iterator for ErrorIterator {
	type Item = ErrorData;

	fn next(&mut self) -> Option<Self::Item> {
		if let Some((data, span)) = self.list.get(self.next).cloned() {
			self.next += 1;
			Some(ErrorData { data, span })
		} else {
			None
		}
	}
}

impl WithRepr for ErrorData {
	fn output(&self, mode: ReprMode, format: ReprFormat, output: &mut dyn std::fmt::Write) -> std::fmt::Result {
		// TODO: properly use the location
		self.data.output(mode, format, output)
	}
}

//====================================================================================================================//
// Traits
//====================================================================================================================//

impl Error for Errors {}

has_traits!(Errors: WithRepr);

impl WithRepr for Errors {
	fn output(&self, mode: ReprMode, format: ReprFormat, mut output: &mut dyn std::fmt::Write) -> std::fmt::Result {
		let debug = mode.is_debug();
		if self.len() == 0 {
			if debug {
				write!(output, "<NoErrors>")
			} else {
				write!(output, "")
			}
		} else {
			if debug {
				write!(output, "<Errors")?;
			} else {
				write!(output, "Errors:\n")?;
			}

			{
				let mut output = output.indented();
				for (n, it) in self.iter().enumerate() {
					write!(output, "\n[{}]", n + 1)?;
					write!(output, " ")?;
					it.output(ReprMode::Display, format, &mut output)?;
				}
			}

			if debug {
				write!(output, "\n>")?;
			} else {
				write!(output, "\n")?;
			}
			Ok(())
		}
	}
}

fmt_from_repr!(Errors);

//====================================================================================================================//
// Conversion
//====================================================================================================================//

impl From<std::io::Error> for Errors {
	fn from(value: std::io::Error) -> Self {
		Errors::from(format!("io error: {value}"))
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
		let mut errors = Errors::default();
		errors.add("some error 1");
		errors.add("some error 2".to_string());
		errors.add("error A");
		errors.add("error B");
		errors.add("error C\n    with some detail");

		let expected = vec![
			"Errors:",
			"",
			"    [1] some error 1",
			"    [2] some error 2",
			"    [3] error A",
			"    [4] error B",
			"    [5] error C",
			"        with some detail",
			"",
		];
		let expected = expected.join("\n");
		let actual = errors.to_string();
		assert_eq!(actual, expected);
	}
}
