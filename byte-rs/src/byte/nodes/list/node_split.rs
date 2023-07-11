use super::*;

pub trait NodeListSplit {
	fn is_split(&self, node: &Node) -> bool;

	fn new_node(&self, nodes: NodeList) -> Result<Node>;
}

impl NodeList {
	pub fn can_split<T: NodeListSplit>(&self, op: &T) -> bool {
		self.contains(|x| op.is_split(x))
	}

	pub fn split<T: NodeListSplit>(&mut self, op: &T, context: &mut EvalContext) -> Result<()> {
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
}
