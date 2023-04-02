use std::fmt::{Debug, Display};

use crate::core::context::*;
use crate::core::input::*;

pub trait ErrorInfo: Display + Debug + 'static {}

#[derive(Clone)]
pub struct Error {
	info: &'static dyn ErrorInfo,
	span: Span,
}

impl Error {
	pub fn span(&self) -> Span {
		self.span.clone()
	}
}

impl std::fmt::Display for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{} at {}", self.info, self.span())
	}
}

impl std::fmt::Debug for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"[error: {} at {}@{}]",
			self.info,
			self.span(),
			self.span().src()
		)
	}
}

#[derive(Copy, Clone)]
#[allow(unused)]
pub struct ErrorList {
	ctx: Context,
	head: Option<&'static ErrorNode>,
}

#[allow(unused)]
impl ErrorList {
	pub fn new(ctx: Context) -> ErrorList {
		ErrorList { ctx, head: None }
	}

	pub fn empty(&self) -> bool {
		self.head.is_none()
	}

	pub fn at<T: ErrorInfo>(&mut self, span: Span, info: T) {
		let node = ErrorNode {
			info: Box::new(info),
			span,
			prev: self.head,
		};
		let node = self.ctx.save(node);
		self.head = Some(node);
	}

	pub fn list(&self) -> Vec<Error> {
		let mut list = Vec::new();
		if let Some(node) = self.head {
			node.append_to(&mut list);
		}
		list
	}
}

struct ErrorNode {
	info: Box<dyn ErrorInfo>,
	span: Span,
	prev: Option<&'static ErrorNode>,
}

impl ErrorNode {
	fn append_to(&'static self, list: &mut Vec<Error>) {
		if let Some(prev) = self.prev {
			prev.append_to(list);
		}
		list.push(Error {
			info: &*self.info,
			span: self.span.clone(),
		})
	}
}
