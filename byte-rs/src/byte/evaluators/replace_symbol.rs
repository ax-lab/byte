use super::*;

pub struct ReplaceSymbol(pub Symbol, pub fn(Span) -> Node);

impl IsNodeOperator for ReplaceSymbol {
	fn predicate(&self, node: &Node) -> bool {
		node.symbol().as_ref() == Some(&self.0)
	}

	fn apply(&self, nodes: &mut NodeList, context: &mut EvalContext) -> Result<()> {
		let _ = context;
		nodes.replace(|node| {
			if node.symbol().as_ref() == Some(&self.0) {
				let span = node.span().clone();
				let node = (self.1)(span);
				Some(node)
			} else {
				None
			}
		});
		Ok(())
	}
}
