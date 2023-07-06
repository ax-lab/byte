use super::*;

pub struct ReplaceSymbol(pub Symbol, pub fn(Span) -> Node, pub Precedence);

impl IsOperator for ReplaceSymbol {
	fn precedence(&self) -> Precedence {
		self.2
	}

	fn predicate(&self, node: &Node) -> bool {
		node.symbol().as_ref() == Some(&self.0)
	}

	fn apply(&self, context: &mut OperatorContext, errors: &mut Errors) {
		let _ = errors;
		let nodes = context.nodes();
		nodes.replace(|node| {
			if node.symbol().as_ref() == Some(&self.0) {
				let span = node.span().clone();
				let node = (self.1)(span);
				Some(node)
			} else {
				None
			}
		});
	}
}
