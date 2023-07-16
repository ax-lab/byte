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

	pub fn check(self) -> Result<()> {
		if self.len() > 0 {
			Err(self)
		} else {
			Ok(())
		}
	}

	pub fn at_pos(mut self, new_span: Span) -> Self {
		let list = Arc::make_mut(&mut self.list);
		for (_, span) in list.iter_mut() {
			if *span == Span::default() {
				*span = new_span.clone()
			}
		}
		self
	}

	pub fn at<T: Into<String>>(mut self, context: T, span: Span) -> Self {
		let context = context.into();
		match self.list.len() {
			0 => {
				let list = Arc::make_mut(&mut self.list);
				list.push_back((context, span));
				self
			}
			1 => {
				let list = Arc::make_mut(&mut self.list);
				list[0].0 = format!("{context}: {}", list[0].0);
				if list[0].1 == Span::default() {
					list[0].1 = span;
				}
				self
			}
			_ => {
				let mut output = context;
				{
					let mut output = output.indented();
					let _ = write!(output, ":\n");
					for (n, it) in self.iter().enumerate() {
						let _ = write!(output, "\n[{}]", n + 1);
						let _ = write!(output, " {it}");
					}
				}
				Errors::from(output, span)
			}
		}
	}

	pub fn from<T: Into<String>>(error: T, span: Span) -> Self {
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

pub trait ResultChain {
	type Result;

	fn and(self, other: Self) -> Self;
	fn or(self, other: Self) -> Self;
	fn unless(self, errors: Errors) -> Self;

	fn at_pos(self, span: Span) -> Self;
	fn at<T: Into<String>>(self, context: T, span: Span) -> Self;

	fn take_errors(self, errors: &mut Errors);
}

impl<T> ResultChain for Result<T> {
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

	fn unless(self, errors: Errors) -> Self {
		if !errors.empty() {
			match self {
				Ok(..) => Err(errors),
				Err(mut err) => {
					err.append(&errors);
					Err(err)
				}
			}
		} else {
			self
		}
	}

	fn at_pos(self, span: Span) -> Self {
		match self {
			ok @ Ok(_) => ok,
			Err(errors) => Err(errors.at_pos(span)),
		}
	}

	fn at<U: Into<String>>(self, context: U, span: Span) -> Self {
		match self {
			ok @ Ok(_) => ok,
			Err(errors) => Err(errors.at(context, span)),
		}
	}

	fn take_errors(self, errors: &mut Errors) {
		match self {
			Err(err) => errors.append(&err),
			_ => (),
		}
	}
}

pub trait ResultChainDefault {
	type Result;

	fn handle(self, errors: &mut Errors) -> Self::Result;
}

impl<T: Default> ResultChainDefault for Result<T> {
	type Result = T;

	fn handle(self, errors: &mut Errors) -> Self::Result {
		match self {
			Ok(value) => value,
			Err(errs) => {
				errors.append(&errs);
				Self::Result::default()
			}
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
		Errors::from(format!("io error: {value}"), Span::default())
	}
}

impl From<std::string::FromUtf8Error> for Errors {
	fn from(value: std::string::FromUtf8Error) -> Self {
		Errors::from(format!("utf8 error: {value}"), Span::default())
	}
}

impl From<std::fmt::Error> for Errors {
	fn from(value: std::fmt::Error) -> Self {
		Errors::from(format!("{value}"), Span::default())
	}
}

impl From<std::num::ParseIntError> for Errors {
	fn from(value: std::num::ParseIntError) -> Self {
		Errors::from(format!("{value}"), Span::default())
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
