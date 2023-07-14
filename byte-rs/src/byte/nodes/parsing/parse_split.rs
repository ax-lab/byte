use super::*;

pub trait ParseSplitBy {
	fn skip_empty(&self) -> bool;

	fn is_split(&self, node: &Node) -> bool;

	fn new_node(&self, ctx: &mut EvalContext, node: Node) -> Result<Node>;
}

pub trait ParseSplitSequence {
	fn is_split(&self, node: &Node) -> bool;

	fn new_node(&self, ctx: &mut EvalContext, node: Vec<Node>, span: Span) -> Result<Node>;
}

impl Node {
	pub fn can_split<T: ParseSplitBy>(&self, op: &T) -> bool {
		self.contains(|x| op.is_split(x))
	}

	pub fn can_split_sequence<T: ParseSplitSequence>(&self, op: &T) -> bool {
		self.contains(|x| op.is_split(x))
	}

	pub fn split<T: ParseSplitBy>(&mut self, ctx: &mut EvalContext, op: &T) -> Result<()> {
		let scope = self.scope();
		let mut new_nodes = Vec::new();
		let mut line = Vec::new();

		let mut has_separator = false;
		for it in self.iter() {
			if op.is_split(&it) {
				has_separator = true;
				let node = Node::raw(std::mem::take(&mut line), scope.handle());
				if node.len() == 0 && op.skip_empty() {
					continue;
				}
				let node = op.new_node(ctx, node)?;
				node.get_dependencies(|list| ctx.add_new_node(list));
				new_nodes.push(node);
			} else {
				line.push(it.clone());
			}
		}

		if line.len() > 0 {
			let node = Node::raw(std::mem::take(&mut line), scope.handle());
			let node = op.new_node(ctx, node)?;
			node.get_dependencies(|list| ctx.add_new_node(list));
			new_nodes.push(node);
		}

		if has_separator {
			self.replace_all(new_nodes);
		}
		Ok(())
	}

	pub fn split_sequence<T: ParseSplitSequence>(&mut self, ctx: &mut EvalContext, op: &T) -> Result<()> {
		let scope = self.scope();
		let mut new_nodes = Vec::new();
		let mut line = Vec::new();

		let mut has_splits = false;
		for it in self.iter() {
			if op.is_split(&it) {
				let node = std::mem::take(&mut line);
				let node = Node::raw(node, scope.handle());
				ctx.add_new_node(&node);
				new_nodes.push(node);
				has_splits = true;
			} else {
				line.push(it.clone());
			}
		}

		if has_splits {
			if line.len() > 0 {
				let node = std::mem::take(&mut line);
				let node = Node::raw(node, scope.handle());
				ctx.add_new_node(&node);
				new_nodes.push(node);
			}

			let node = op.new_node(ctx, new_nodes, self.span())?;
			self.replace_all(vec![node]);
		}

		Ok(())
	}
}
