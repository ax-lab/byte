use std::io::Write;

use crate::core::*;

pub trait IsNode: IsValue {}

#[derive(Clone, Eq, PartialEq)]
pub struct Node {
	data: Value,
	span: Option<Span>,
}

impl Node {
	pub fn from<T: IsNode>(node: T) -> Self {
		Value::from(node).into()
	}

	pub fn value(&self) -> &dyn IsNode {
		get_trait!(self, IsNode).unwrap()
	}

	pub fn at(mut self, span: Option<Span>) -> Node {
		self.span = span.or(self.span);
		self
	}

	pub fn get<T: IsNode>(&self) -> Option<&T> {
		self.data.get()
	}

	pub fn is<T: IsNode>(&self, value: T) -> bool {
		self.data == Value::from(value)
	}
}

impl From<Value> for Node {
	fn from(value: Value) -> Self {
		assert!(value.is_node());
		Node {
			data: value,
			span: None,
		}
	}
}

impl Value {
	pub fn is_node(&self) -> bool {
		self.as_node().is_some()
	}

	pub fn as_node(&self) -> Option<&dyn IsNode> {
		get_trait!(self, IsNode)
	}
}

fmt_from_repr!(Node);

impl HasRepr for Node {
	fn output_repr(&self, output: &mut Repr<'_>) -> std::io::Result<()> {
		self.data.output_repr(output)?;
		if let Some(span) = self.span() {
			write!(output, " @")?;
			span.output_repr(&mut output.display().compact())?;
		}
		Ok(())
	}
}

impl HasTraits for Node {
	fn get_trait(&self, type_id: std::any::TypeId) -> Option<&dyn HasTraits> {
		with_trait!(self, type_id, WithSpan);
		with_trait!(self, type_id, WithEquality);
		self.data.get_trait(type_id)
	}
}

impl WithSpan for Node {
	fn get_span(&self) -> Option<Span> {
		self.data.span().or(self.span.clone())
	}
}
