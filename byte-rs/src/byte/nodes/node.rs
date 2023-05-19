use super::*;

#[derive(Clone)]
pub struct Node {
	data: Value,
}

impl Node {
	pub fn from<T: IsNode>(node: T) -> Self {
		Value::from(node).into()
	}

	pub fn empty() -> Self {
		Self::from(EmptyNode)
	}

	//----------------------------------------------------------------------------------------------------------------//
	// Value
	//----------------------------------------------------------------------------------------------------------------//

	pub fn value(&self) -> &dyn IsNode {
		get_trait!(self, IsNode).unwrap()
	}

	pub fn get<T: IsNode>(&self) -> Option<&T> {
		self.data.get()
	}

	pub fn is<T: IsNode>(&self) -> bool {
		self.data.get::<T>().is_some()
	}

	pub fn get_field<T: IsValue>(&self) -> Option<&T> {
		self.data.get_field()
	}

	pub fn with_field<T: IsValue>(&self, value: T) -> Node {
		let data = self.data.with_field(value);
		Node { data }
	}

	//----------------------------------------------------------------------------------------------------------------//
	// Location
	//----------------------------------------------------------------------------------------------------------------//

	pub fn at(self, span: Option<Span>) -> Node {
		if let Some(span) = span {
			self.with_span(span)
		} else {
			self
		}
	}

	pub fn with_span(mut self, span: Span) -> Node {
		self.data = self.data.with_span(span);
		self
	}

	pub fn span(&self) -> Option<&Span> {
		self.data.get_span()
	}

	pub fn indent(&self) -> usize {
		self.span()
			.map(|x| x.location().indent())
			.unwrap_or_default()
	}

	pub fn line(&self) -> Option<usize> {
		self.span().and_then(|x| x.location().line())
	}

	pub fn format_location(&self, label: &str) -> String {
		self.span()
			.map(|x| x.location().format(label))
			.unwrap_or_default()
	}
}

impl From<Value> for Node {
	fn from(value: Value) -> Self {
		assert!(value.is_node());
		Node { data: value }
	}
}

impl<T: IsNode> From<T> for Node {
	fn from(value: T) -> Self {
		Value::from(value).into()
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
		with_trait!(self, type_id, WithEquality);
		self.data.get_trait(type_id)
	}
}

impl PartialEq for Node {
	fn eq(&self, other: &Self) -> bool {
		self.value().is_equal(&other.data)
	}
}

impl Eq for Node {}

impl Default for Node {
	fn default() -> Self {
		Self::empty()
	}
}

//====================================================================================================================//
// Empty node
//====================================================================================================================//

#[derive(Eq, PartialEq)]
pub struct EmptyNode;

has_traits!(EmptyNode: IsNode);

impl IsNode for EmptyNode {}

impl HasRepr for EmptyNode {
	fn output_repr(&self, output: &mut Repr) -> std::io::Result<()> {
		if output.is_debug() {
			write!(output, "<EmptyNode>")
		} else {
			write!(output, "empty")
		}
	}
}
