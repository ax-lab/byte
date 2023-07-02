use super::*;

pub struct BindOperator;

impl IsOperator for BindOperator {
	fn precedence(&self) -> Precedence {
		Precedence::Bind
	}

	fn predicate(&self, node: &NodeData) -> bool {
		matches!(node.get(), Node::Word(..))
	}

	fn apply(&self, context: &mut OperatorContext, errors: &mut Errors) {
		let mut nodes = context.nodes().clone();
		let scope = nodes.scope();
		nodes.replace(|node| {
			if let Node::Word(name) = node.get() {
				let span = node.span().clone();
				if let Some(index) = scope.lookup(name, Some(node.offset())) {
					let value = Node::Variable(name.clone(), index);
					Some(value.at(span))
				} else {
					let error = format!("undefined symbol `{name}`");
					errors.add_at(error, span);
					None
				}
			} else {
				None
			}
		});
	}
}
