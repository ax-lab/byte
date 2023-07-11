use super::*;

pub struct ReplaceSymbol(pub Symbol, pub fn(Span) -> Node);

impl NodeReplace for ReplaceSymbol {
	fn can_replace(&self, node: &Node) -> bool {
		if let Some(symbol) = node.symbol() {
			symbol == self.0
		} else {
			false
		}
	}

	fn replace(&self, node: &Node, context: &mut EvalContext) -> Result<Option<Node>> {
		let _ = context;
		let new_node = &self.1;
		if let Some(symbol) = node.symbol() {
			if symbol == self.0 {
				Ok(Some(new_node(node.span())))
			} else {
				Ok(None)
			}
		} else {
			Ok(None)
		}
	}
}

impl IsNodeOperator for ReplaceSymbol {
	fn can_apply(&self, nodes: &NodeList) -> bool {
		nodes.can_replace(self)
	}

	fn apply(&self, nodes: &mut NodeList, context: &mut EvalContext) -> Result<()> {
		nodes.replace(self, context)
	}
}
