use std::{collections::VecDeque, error::Error, sync::Arc};

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
// ErrorIterator
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
// Error location
//====================================================================================================================//

/// Trait for relaying source location information.
pub trait WithLocation {
	/// Source file name.
	fn source(&self) -> Option<&str>;

	/// Line number or zero if not available.
	fn line(&self) -> usize;

	/// Column number or zero if not available.
	fn column(&self) -> usize;

	/// True if neither of the source or location are available.
	fn empty(&self) -> bool {
		self.source().is_none() && !self.has_location()
	}

	/// True if line information is available.
	fn has_location(&self) -> bool {
		self.line() > 0
	}

	fn format_short(&self, separator: &str, output: &mut dyn std::fmt::Write) -> std::fmt::Result {
		let line = self.line();
		if line > 0 {
			write!(output, "{separator}{line}")?;
			let column = self.column();
			if column > 0 {
				write!(output, ":{column}")?;
			}
		}
		Ok(())
	}

	fn format(&self, separator: &str, output: &mut dyn std::fmt::Write) -> std::fmt::Result {
		if let Some(src) = self.source() {
			write!(output, "{separator}{src}")?;
			self.format_short(":", output)?;
		} else {
			let line = self.line();
			if line > 0 {
				write!(output, "{separator}line {line}")?;
				let column = self.column();
				if column > 0 {
					write!(output, ":{column}")?;
				}
			}
		}
		Ok(())
	}
}

impl Value {
	pub fn with_location(&self) -> Option<&dyn WithLocation> {
		get_trait!(self, WithLocation)
	}
}

//====================================================================================================================//
// Traits
//====================================================================================================================//

impl Error for Errors {}

impl WithRepr for Errors {
	fn output(
		&self,
		mode: ReprMode,
		format: ReprFormat,
		mut output: &mut dyn std::fmt::Write,
	) -> std::fmt::Result {
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

					let has_location = if let Some(location) = it.with_location() {
						location.format(" at ", &mut output)?;
						true
					} else {
						false
					};

					if has_location {
						let mut output = output.indented();
						write!(output, "\n")?;
						it.output(mode, format, &mut output)?;
					} else {
						write!(output, " ")?;
						it.output(mode, format, &mut output)?;
					}
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

impl std::fmt::Debug for Errors {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		self.fmt_debug(f)
	}
}

impl std::fmt::Display for Errors {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		self.fmt_display(f)
	}
}

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
		const F: &'static str = "input.txt";

		let mut errors = Errors::default();
		errors.add("some error 1");
		errors.add("some error 2".to_string());
		errors.add(Error::at(F, 1, 2, "error A"));
		errors.add(Error::at(F, 3, 4, "error B"));
		errors.add(Error::at("", 5, 6, "error C\n    with some detail"));

		let expected = vec![
			"Errors:",
			"",
			"    [1] some error 1",
			"    [2] some error 2",
			"    [3] at input.txt:1:2",
			"        error A",
			"    [4] at input.txt:3:4",
			"        error B",
			"    [5] at line 5:6",
			"        error C",
			"            with some detail",
			"",
		];
		let expected = expected.join("\n");
		let actual = errors.to_string();
		assert_eq!(actual, expected);
	}

	struct Error {
		error: &'static str,
		input: &'static str,
		line: usize,
		column: usize,
	}

	has_traits!(Error: WithLocation, WithRepr);

	impl Error {
		pub fn at(input: &'static str, line: usize, column: usize, error: &'static str) -> Self {
			Self {
				input,
				line,
				column,
				error,
			}
		}
	}

	impl WithLocation for Error {
		fn source(&self) -> Option<&str> {
			if self.input.len() > 0 {
				Some(self.input)
			} else {
				None
			}
		}

		fn line(&self) -> usize {
			self.line
		}

		fn column(&self) -> usize {
			self.column
		}
	}

	impl WithRepr for Error {
		fn output(
			&self,
			_mode: ReprMode,
			_format: ReprFormat,
			output: &mut dyn Write,
		) -> std::fmt::Result {
			write!(output, "{}", self.error)
		}
	}
}
