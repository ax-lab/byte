use super::*;

pub trait NodeSplitBy {
	fn is_split(&self, node: &Node) -> bool;

	fn new_node(&self, nodes: NodeList) -> Result<Node>;
}

pub trait NodeSplitSequence {
	fn is_split(&self, node: &Node) -> bool;

	fn new_node(&self, nodes: Vec<NodeList>, span: Span) -> Result<Node>;
}

impl NodeList {
	pub fn can_split<T: NodeSplitBy>(&self, op: &T) -> bool {
		self.contains(|x| op.is_split(x))
	}

	pub fn can_split_sequence<T: NodeSplitSequence>(&self, op: &T) -> bool {
		self.contains(|x| op.is_split(x))
	}

	pub fn split<T: NodeSplitBy>(&mut self, op: &T, context: &mut EvalContext) -> Result<()> {
		let scope = self.scope();
		let mut new_nodes = Vec::new();
		let mut line = Vec::new();

		for it in self.iter() {
			if op.is_split(&it) {
				let nodes = NodeList::new(scope.handle(), std::mem::take(&mut line));
				let node = op.new_node(nodes)?;
				node.bit().get_dependencies(|list| context.resolve_nodes(list));
				new_nodes.push(node);
			} else {
				line.push(it.clone());
			}
		}

		if line.len() > 0 {
			let nodes = NodeList::new(scope.handle(), std::mem::take(&mut line));
			let node = op.new_node(nodes)?;
			node.bit().get_dependencies(|list| context.resolve_nodes(list));
			new_nodes.push(node);
		}

		if new_nodes.len() > 0 {
			self.replace_all(new_nodes);
		}
		Ok(())
	}

	pub fn split_sequence<T: NodeSplitSequence>(&mut self, op: &T, context: &mut EvalContext) -> Result<()> {
		let scope = self.scope();
		let mut new_nodes = Vec::new();
		let mut line = Vec::new();

		let mut has_splits = false;
		for it in self.iter() {
			if op.is_split(&it) {
				let nodes = std::mem::take(&mut line);
				let nodes = NodeList::new(scope.handle(), nodes);
				context.resolve_nodes(&nodes);
				new_nodes.push(nodes);
				has_splits = true;
			} else {
				line.push(it.clone());
			}
		}

		if has_splits {
			if line.len() > 0 {
				let nodes = std::mem::take(&mut line);
				let nodes = NodeList::new(scope.handle(), nodes);
				context.resolve_nodes(&nodes);
				new_nodes.push(nodes);
			}

			let node = op.new_node(new_nodes, self.span())?;
			self.replace_all(vec![node]);
		}

		Ok(())
	}
}
