use super::*;

pub trait ParseTernary {
	fn delimiters(&self) -> (&Symbol, &Symbol);

	fn new_node(&self, ctx: &mut EvalContext, a: NodeList, b: NodeList, c: NodeList, span: Span) -> Result<Node>;
}

impl NodeList {
	pub fn has_ternary<T: ParseTernary>(&self, op: &T) -> bool {
		let (sta, end) = op.delimiters();
		let nodes = self.iter();
		let mut nodes = nodes.skip_while(|x| x.symbol().as_ref() != Some(sta));
		if let Some(..) = nodes.next() {
			let mut nodes = nodes.skip_while(|x| x.symbol().as_ref() != Some(end));
			nodes.next().is_some()
		} else {
			false
		}
	}

	pub fn parse_ternary<T: ParseTernary>(&mut self, ctx: &mut EvalContext, op: &T) -> Result<()> {
		let (sta, end) = op.delimiters();
		let mut nodes = self.data.nodes.write().unwrap();
		for i in (0..nodes.len()).rev() {
			if nodes[i].has_symbol(sta) {
				for j in i + 1..nodes.len() {
					if nodes[j].has_symbol(end) {
						let a = nodes[0..i].to_vec();
						let b = nodes[i + 1..j].to_vec();
						let c = nodes[j + 1..].to_vec();
						let a = NodeList::new(self.scope_handle(), a);
						let b = NodeList::new(self.scope_handle(), b);
						let c = NodeList::new(self.scope_handle(), c);
						let node = op.new_node(ctx, a, b, c, self.span())?;
						node.get_dependencies(|list| ctx.resolve_nodes(list));

						let nodes = Arc::make_mut(&mut nodes);
						*nodes = vec![node];
						return Ok(());
					}
				}
			}
		}
		Ok(())
	}
}
