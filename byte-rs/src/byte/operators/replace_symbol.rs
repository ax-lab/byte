use super::*;

pub struct ReplaceSymbol(pub Symbol, pub fn(Span) -> Node);

impl Evaluator for ReplaceSymbol {
	fn predicate(&self, node: &Node) -> bool {
		node.symbol().as_ref() == Some(&self.0)
	}

	fn apply(&self, scope: &Scope, nodes: &mut Vec<Node>, context: &mut EvalContext) -> Result<bool> {
		let _ = (scope, context);
		let changed = Nodes::replace(nodes, |node| {
			if node.symbol().as_ref() == Some(&self.0) {
				let span = node.span().clone();
				let node = (self.1)(span);
				Some(node)
			} else {
				None
			}
		});
		Ok(changed)
	}
}
