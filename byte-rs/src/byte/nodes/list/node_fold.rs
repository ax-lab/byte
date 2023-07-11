use super::*;

pub trait NodeFold {
	fn fold_at(&self, nodes: &NodeList) -> Option<usize>;

	fn new_node(&self, ctx: &mut EvalContext, lhs: NodeList, rhs: NodeList, span: Span) -> Result<Node>;
}

impl NodeList {
	pub fn can_fold<T: NodeFold>(&self, op: &T) -> bool {
		op.fold_at(self).is_some()
	}

	pub fn fold<T: NodeFold>(&mut self, op: &T, ctx: &mut EvalContext) -> Result<()> {
		if let Some(index) = op.fold_at(self) {
			let span = self.span();
			let lhs = self.slice(..index);
			let rhs = self.slice(index + 1..);
			let node = op.new_node(ctx, lhs, rhs, span)?;
			node.get_dependencies(|list| ctx.resolve_nodes(list));
			self.write_res(|nodes| {
				*nodes = vec![node];
				Ok(true)
			})?;
		}
		Ok(())
	}
}
