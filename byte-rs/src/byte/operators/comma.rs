use super::*;

pub struct CommaOperator;

impl IsOperator for CommaOperator {
	fn precedence(&self) -> Precedence {
		Precedence::Comma
	}

	fn predicate(&self, node: &Node) -> bool {
		if let Node::Symbol(name) = node {
			name == ","
		} else {
			false
		}
	}

	fn apply(&self, context: &mut OperatorContext, errors: &mut Errors) {
		let _ = errors;

		let mut nodes = context.nodes().clone();
		let span = nodes.span();
		let items = nodes.split_by_items(|n| self.predicate(n.get()));

		for it in items.iter() {
			context.resolve_nodes(it);
		}

		let node = Node::Sequence(items).at(span);
		nodes.replace_all(vec![node]);
	}
}
