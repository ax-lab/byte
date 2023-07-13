use super::*;

pub trait ParseKeyword {
	fn symbol(&self) -> &Symbol;

	fn new_node(&self, ctx: &mut EvalContext, args: NodeList, span: Span) -> Result<Node>;
}

impl NodeList {
	pub fn has_keyword<T: ParseKeyword>(&self, op: &T) -> bool {
		self.is_keyword(0, op.symbol())
	}

	pub fn parse_keyword<T: ParseKeyword>(&mut self, ctx: &mut EvalContext, op: &T) -> Result<()> {
		let args = self.slice(1..);
		let span = self.span();
		let node = op.new_node(ctx, args, span)?;
		node.get_dependencies(|list| ctx.add_segment(list));
		self.replace_all(vec![node]);
		Ok(())
	}
}
