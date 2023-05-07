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

	pub fn is<T: IsNode>(&self) -> bool {
		self.data.get::<T>().is_some()
	}

	pub fn indent(&self) -> usize {
		self.span().map(|x| x.start().indent()).unwrap()
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

//====================================================================================================================//
// Repr
//====================================================================================================================//

fmt_from_repr!(Node);

impl HasRepr for Node {
	fn output_repr(&self, output: &mut Repr<'_>) -> std::io::Result<()> {
		self.data.output_repr(output)?;
		if let Some(span) = self.span() {
			if output.is_debug() {
				write!(output, " @")?;
				span.output_repr(&mut output.display().minimal())?;
			}
		}
		Ok(())
	}
}

impl Node {
	pub fn output_repr_start(
		output: &mut Repr<'_>,
		debug: &str,
		display: &str,
	) -> std::io::Result<()> {
		if output.is_debug() {
			write!(output, "{debug}")
		} else {
			write!(output, "{display}")
		}
	}

	pub fn output_repr_list(
		output: &mut Repr<'_>,
		list: &[Node],
		sep: &str,
	) -> std::io::Result<()> {
		let sep = if sep.len() > 0 { sep } else { " " };
		if list.len() > 0 {
			let mut output = output.indented();
			if !output.is_compact() {
				write!(output, "\n")?;
			} else {
				write!(output, " ")?;
			}
			for (i, it) in list.iter().enumerate() {
				if output.is_debug() && !output.is_compact() {
					write!(output, "[{i}] ")?;
				}
				if output.is_compact() && i > 0 {
					write!(output, "{sep}")?;
				}
				it.output_repr(&mut output)?;
				if !output.is_compact() {
					write!(output, "\n")?;
				}
			}
		}
		Ok(())
	}

	pub fn output_repr_end(
		output: &mut Repr<'_>,
		debug: &str,
		display: &str,
	) -> std::io::Result<()> {
		if output.is_debug() {
			write!(output, "{debug}")
		} else {
			write!(output, "{display}")
		}
	}
}

//====================================================================================================================//
// Traits
//====================================================================================================================//

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

impl Node {
	pub fn span_for_list(nodes: &[Node]) -> Option<Span> {
		let span = Span::from_range(
			nodes.first().and_then(|x| x.span()),
			nodes.last().and_then(|x| x.span()),
		);
		span.or_else(|| {
			nodes.first().and_then(|x| {
				x.span().map(|x| {
					let start = x.start();
					Span::new(&start, &start)
				})
			})
		})
	}
}
