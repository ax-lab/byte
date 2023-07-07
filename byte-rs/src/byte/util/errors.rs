use std::{collections::VecDeque, error::Error, sync::Arc};

use super::*;

pub const MAX_ERRORS: usize = 32;

/// List of errors.
#[derive(Clone, Default)]
pub struct Errors {
	list: Arc<VecDeque<(String, Span)>>,
}

impl Errors {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn from<T: Into<String>>(error: T) -> Self {
		let mut errors = Self::new();
		errors.add(error, Span::default());
		errors
	}

	pub fn from_at<T: Into<String>>(error: T, span: Span) -> Self {
		let mut errors = Self::new();
		errors.add(error, span);
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

	pub fn add<T: Into<String>>(&mut self, error: T, span: Span) {
		let list = Arc::make_mut(&mut self.list);
		list.push_back((error.into(), span));
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
	data: String,
	span: Span,
}

pub struct ErrorIterator {
	next: usize,
	list: Arc<VecDeque<(String, Span)>>,
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

impl Display for ErrorData {
	fn fmt(&self, output: &mut Formatter<'_>) -> std::fmt::Result {
		if let Some(location) = self.span.location() {
			write!(output, "at {location}: ")?;
		}
		write!(output, "{}", self.data)
	}
}

//====================================================================================================================//
// Result
//====================================================================================================================//

pub trait ResultExtension {
	type Result;

	fn and(self, other: Self) -> Self;
	fn or(self, other: Self) -> Self;
	fn handle(self, error: &mut Errors) -> Self::Result;
	fn unless(self, errors: Errors) -> Self;
}

impl<T: Default> ResultExtension for Result<T> {
	type Result = T;

	fn and(self, other: Self) -> Self {
		match self {
			Ok(..) => other,
			Err(errors) => match other {
				Ok(..) => Err(errors),
				Err(other) => {
					let mut errors = errors;
					errors.append(&other);
					Err(errors)
				}
			},
		}
	}

	fn or(self, other: Self) -> Self {
		match self {
			Ok(a) => Ok(a),
			Err(errors) => match other {
				Ok(b) => Ok(b),
				Err(other) => {
					let mut errors = errors;
					errors.append(&other);
					Err(errors)
				}
			},
		}
	}

	fn handle(self, errors: &mut Errors) -> Self::Result {
		match self {
			Ok(value) => value,
			Err(errs) => {
				errors.append(&errs);
				Self::Result::default()
			}
		}
	}

	fn unless(self, errors: Errors) -> Self {
		if !errors.empty() {
			self.and(Err(errors))
		} else {
			self
		}
	}
}

//====================================================================================================================//
// Traits
//====================================================================================================================//

impl Error for Errors {}

impl Display for Errors {
	fn fmt(&self, output: &mut Formatter<'_>) -> std::fmt::Result {
		if self.len() == 0 {
			write!(output, "")
		} else {
			write!(output, "Errors:\n")?;
			{
				let mut output = output.indented();
				for (n, it) in self.iter().enumerate() {
					write!(output, "\n[{}]", n + 1)?;
					write!(output, " {it}")?;
				}
			}
			write!(output, "\n")?;
			Ok(())
		}
	}
}

impl Debug for Errors {
	fn fmt(&self, output: &mut Formatter<'_>) -> std::fmt::Result {
		if self.len() == 0 {
			write!(output, "<NoErrors>")
		} else {
			write!(output, "<Errors")?;
			{
				let mut output = output.indented();
				for (n, it) in self.iter().enumerate() {
					write!(output, "\n[{}]", n + 1)?;
					write!(output, " {it}")?;
				}
			}
			write!(output, "\n>")?;
			Ok(())
		}
	}
}

//====================================================================================================================//
// Conversion
//====================================================================================================================//

impl From<std::io::Error> for Errors {
	fn from(value: std::io::Error) -> Self {
		Errors::from(format!("io error: {value}"))
	}
}

impl From<std::string::FromUtf8Error> for Errors {
	fn from(value: std::string::FromUtf8Error) -> Self {
		Errors::from(format!("utf8 error: {value}"))
	}
}

impl From<std::fmt::Error> for Errors {
	fn from(value: std::fmt::Error) -> Self {
		Errors::from(format!("{value}"))
	}
}

impl From<std::num::ParseIntError> for Errors {
	fn from(value: std::num::ParseIntError) -> Self {
		Errors::from(format!("{value}"))
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
		errors.add("some error 1", Span::default());
		errors.add("some error 2".to_string(), Span::default());
		errors.add("error A", Span::default());
		errors.add("error B", Span::default());
		errors.add("error C\n    with some detail", Span::default());

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
