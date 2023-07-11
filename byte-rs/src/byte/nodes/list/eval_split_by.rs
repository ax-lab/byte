use super::*;

pub trait EvalSplitBy {
	fn is_split(&self, node: &Node) -> bool;

	fn new_node(&self, nodes: NodeList) -> Result<Node>;
}

impl<T: EvalSplitBy> IsEvaluator for T {
	fn predicate(&self, node: &Node) -> bool {
		self.is_split(node)
	}

	fn apply(&self, nodes: &mut NodeList, context: &mut EvalContext) -> Result<()> {
		let scope = nodes.scope();
		let mut new_nodes = Vec::new();
		let mut line = Vec::new();

		for it in nodes.iter() {
			if self.is_split(&it) {
				let nodes = NodeList::new(scope.handle(), std::mem::take(&mut line));
				let node = self.new_node(nodes)?;
				node.bit().get_dependencies(|list| context.resolve_nodes(list));
				new_nodes.push(node);
			} else {
				line.push(it.clone());
			}
		}

		if line.len() > 0 {
			let nodes = NodeList::new(scope.handle(), std::mem::take(&mut line));
			let node = self.new_node(nodes)?;
			node.bit().get_dependencies(|list| context.resolve_nodes(list));
			new_nodes.push(node);
		}

		if new_nodes.len() > 0 {
			nodes.replace_all(new_nodes);
		}
		Ok(())
	}
}
