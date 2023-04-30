use std::{
	fmt::{Debug, Display},
	sync::Arc,
};

use crate::core::input::*;

use super::*;

/// Trait for any type that can be used as an [`Error`].
pub trait IsError: IsValue {
	fn output(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
}

impl IsError for String {
	fn output(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{self}")
	}
}

/// Type for any compilation error.
#[derive(Clone)]
pub struct Error {
	info: Arc<dyn IsError>,
	span: Option<Span>,
}

impl Error {
	pub fn new<T: IsError>(info: T) -> Self {
		Error {
			span: None,
			info: Arc::new(info),
		}
	}

	pub fn span(&self) -> Option<Span> {
		self.span.clone()
	}

	pub fn at(mut self, span: Span) -> Self {
		self.span = Some(span);
		self
	}
}

impl Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.info.output(f)?;
		if let Some(span) = self.span() {
			write!(f, "\n")?;
			write!(f, "       (at {}:{})", span.src(), span)?;
		}
		Ok(())
	}
}

impl Debug for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "[error: {}]", self)
	}
}

/// List of [`Error`].
#[derive(Clone)]
pub struct ErrorList {
	head: Option<Arc<ErrorNode>>,
}

impl Default for ErrorList {
	fn default() -> Self {
		Self::new()
	}
}

impl Debug for ErrorList {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		use std::io::Write;

		if self.empty() {
			write!(f, "<no errors>")
		} else {
			let mut repr = Repr::new(ReprMode::Debug, ReprFormat::Full);
			let _ = write!(repr, "<Errors:\n");
			{
				let mut repr = repr.indented();
				for (i, it) in self.list().into_iter().enumerate() {
					let _ = write!(repr, "[{}] ", i + 1);
					if let Some(span) = it.span() {
						let _ = write!(repr, " at {}", span);
					}
					let mut repr = repr.indented();
					let _ = write!(repr, "\n{it}\n");
				}
			}
			let _ = write!(repr, ">");
			write!(f, "{repr}")
		}
	}
}

#[allow(unused)]
impl ErrorList {
	pub fn new() -> ErrorList {
		ErrorList { head: None }
	}

	pub fn empty(&self) -> bool {
		self.head.is_none()
	}

	pub fn append(&mut self, errors: ErrorList) {
		for err in errors.list().into_iter() {
			self.add_error(err);
		}
	}

	pub fn add<T: ToString>(&mut self, error: T) {
		self.add_error(Error::new(error.to_string()))
	}

	pub fn add_error(&mut self, error: Error) {
		let prev = std::mem::take(&mut self.head);
		let node = ErrorNode { error, prev };
		self.head = Some(Arc::new(node));
	}

	pub fn list(&self) -> Vec<Error> {
		let mut list = Vec::new();
		if let Some(node) = &self.head {
			node.append_to(&mut list);
		}
		list
	}

	pub fn at<T: ToString>(&mut self, span: Option<Span>, msg: T) {
		let error = Error::new(msg.to_string());
		let error = if let Some(span) = span {
			error.at(span)
		} else {
			error
		};
		self.add_error(error)
	}
}

struct ErrorNode {
	error: Error,
	prev: Option<Arc<ErrorNode>>,
}

impl ErrorNode {
	fn append_to(&self, output: &mut Vec<Error>) {
		if let Some(prev) = &self.prev {
			prev.append_to(output);
		}
		output.push(self.error.clone());
	}
}
