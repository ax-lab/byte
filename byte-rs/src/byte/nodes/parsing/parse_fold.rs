use super::*;

pub trait ParseFold {
	fn fold_at(&self, node: &Node) -> Option<usize>;

	fn new_node(&self, ctx: &mut OperatorContext, lhs: Node, rhs: Node, span: Span) -> Result<Node>;
}

impl Node {
	pub fn can_fold<T: ParseFold>(&self, op: &T) -> bool {
		op.fold_at(self).is_some()
	}

	pub fn fold<T: ParseFold>(&mut self, ctx: &mut OperatorContext, op: &T) -> Result<()> {
		if let Some(index) = op.fold_at(self) {
			let span = self.span();
			let lhs = self.slice(..index);
			let rhs = self.slice(index + 1..);
			let node = op.new_node(ctx, lhs, rhs, span)?;
			node.get_dependencies(|list| ctx.add_new_node(list));
			self.rewrite_res(|nodes| {
				*nodes = vec![node];
				Ok(true)
			})?;
		}
		Ok(())
	}
}
