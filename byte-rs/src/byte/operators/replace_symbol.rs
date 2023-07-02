use super::*;

pub struct ReplaceSymbol(pub Name, pub Node, pub Precedence);

impl IsOperator for ReplaceSymbol {
	fn precedence(&self) -> Precedence {
		self.2
	}

	fn predicate(&self, node: &NodeData) -> bool {
		node.name().as_ref() == Some(&self.0)
	}

	fn apply(&self, context: &mut OperatorContext, errors: &mut Errors) {
		let _ = errors;
		let nodes = context.nodes();
		nodes.replace(|node| {
			if node.name().as_ref() == Some(&self.0) {
				let span = node.span().clone();
				let node = self.1.clone();
				Some(node.at(span))
			} else {
				None
			}
		});
	}
}
