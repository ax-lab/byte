use super::*;

pub trait NodeKeyword {
	fn symbol(&self) -> &Symbol;

	fn new_node(&self, args: NodeList, span: Span) -> Result<Node>;
}

impl NodeList {
	pub fn has_keyword<T: NodeKeyword>(&self, op: &T) -> bool {
		self.is_keyword(0, op.symbol())
	}

	pub fn parse_keyword<T: NodeKeyword>(&mut self, op: &T, context: &mut EvalContext) -> Result<()> {
		let args = self.slice(1..);
		let span = self.span();
		let node = op.new_node(args, span)?;
		node.get_dependencies(|list| context.resolve_nodes(list));
		self.replace_all(vec![node]);
		Ok(())
	}
}
