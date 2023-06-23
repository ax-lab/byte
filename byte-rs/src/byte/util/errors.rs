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
		Errors { list: Arc::new(list) }
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
		list.push_back(Value::from(error));
	}

	pub fn add_at<T: IsValue>(&mut self, error: T, span: Option<Span>) {
		if let Some(span) = span {
			let inner = Value::from(error);
			self.add(ErrorWithSpan { inner, span })
		} else {
			self.add(error)
		}
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

					let has_location = if let Some(span) = it.span() {
						span.format_full(" at ", &mut output)?;
						true
					} else {
						false
					};

					if has_location {
						let mut output = output.indented();
						write!(output, "\n")?;
						it.output(ReprMode::Display, format, &mut output)?;
					} else {
						write!(output, " ")?;
						it.output(ReprMode::Display, format, &mut output)?;
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

fmt_from_repr!(Errors);

impl std::ops::Index<usize> for Errors {
	type Output = Value;

	fn index(&self, index: usize) -> &Self::Output {
		&self.list[index]
	}
}

//====================================================================================================================//
// Error with span
//====================================================================================================================//

struct ErrorWithSpan {
	inner: Value,
	span: Span,
}

impl HasTraits for ErrorWithSpan {
	fn type_name(&self) -> &'static str {
		self.inner.type_name()
	}

	fn get_trait(&self, type_id: std::any::TypeId) -> Option<&dyn HasTraits> {
		with_trait!(self, type_id, WithSpan);
		self.inner.get_trait(type_id)
	}
}

impl Debug for ErrorWithSpan {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		write!(f, "{:?}", self.inner)
	}
}

impl Display for ErrorWithSpan {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		write!(f, "{}", self.inner)
	}
}

impl WithSpan for ErrorWithSpan {
	fn span(&self) -> Option<&Span> {
		Some(&self.span)
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

//====================================================================================================================//
// Tests
//====================================================================================================================//

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_output() {
		let p1 = pos(1, 2);
		let p2 = pos(3, 4);
		let p3 = p1.with_pos(5, 6, 0);

		let mut errors = Errors::default();
		errors.add("some error 1");
		errors.add("some error 2".to_string());
		errors.add_at("error A", Some(p1.span()));
		errors.add_at("error B", Some(p3.span_from(&p2)));
		errors.add_at("error C\n    with some detail", Some(p3.span()));

		let expected = vec![
			"Errors:",
			"",
			"    [1] some error 1",
			"    [2] some error 2",
			"    [3] at input.txt:1:2",
			"        error A",
			"    [4] at input.txt:3:4…5:6",
			"        error B",
			"    [5] at input.txt:5:6",
			"        error C",
			"            with some detail",
			"",
		];
		let expected = expected.join("\n");
		let actual = errors.to_string();
		assert_eq!(actual, expected);
	}

	fn pos(line: usize, column: usize) -> Cursor {
		Input::new("input.txt", Vec::new()).with_pos(line, column, 0).start()
	}
}
