use std::fmt::Debug;
use std::rc::Rc;

use crate::core::input::*;

pub trait ErrorInfo: Debug + 'static {
	fn output(&self, f: &mut std::fmt::Formatter<'_>, span: &Span) -> std::fmt::Result;
}

#[derive(Clone)]
pub struct Error {
	info: Rc<dyn ErrorInfo>,
	span: Span,
}

impl Error {
	pub fn new<T: ErrorInfo>(span: Span, info: T) -> Self {
		Error {
			span,
			info: Rc::new(info),
		}
	}

	pub fn span(&self) -> &Span {
		&self.span
	}
}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let span = &self.span;
		write!(f, "error: ")?;
		self.info.output(f, span)?;
		write!(f, "\n")?;
		write!(f, "       (at {}:{})", self.span().src(), self.span())?;
		Ok(())
	}
}

impl std::fmt::Debug for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "[error: {}]", self)
	}
}

#[derive(Clone)]
pub struct ErrorList {
	head: Option<Rc<ErrorNode>>,
}

#[allow(unused)]
impl ErrorList {
	pub fn new() -> ErrorList {
		ErrorList { head: None }
	}

	pub fn empty(&self) -> bool {
		self.head.is_none()
	}

	pub fn add(&mut self, error: Error) {
		let prev = std::mem::take(&mut self.head);
		let node = ErrorNode { error, prev };
		self.head = Some(Rc::new(node));
	}

	pub fn list(&self) -> Vec<Error> {
		let mut list = Vec::new();
		if let Some(node) = &self.head {
			node.append_to(&mut list);
		}
		list
	}
}

struct ErrorNode {
	error: Error,
	prev: Option<Rc<ErrorNode>>,
}

impl ErrorNode {
	fn append_to(&self, output: &mut Vec<Error>) {
		if let Some(prev) = &self.prev {
			prev.append_to(output);
		}
		output.push(self.error.clone());
	}
}
