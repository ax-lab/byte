use std::fmt::Debug;

use crate::core::input::*;

pub trait IsNode: Debug + 'static {}

pub struct Node {
	value: Box<dyn IsNode>,
	span: Span,
}

impl Node {
	pub fn new<T: IsNode>(node: T, span: Span) -> Self {
		Node {
			value: Box::new(node),
			span,
		}
	}

	pub fn span(&self) -> Span {
		self.span.clone()
	}
}

impl Debug for Node {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:?}", self.value)
	}
}
