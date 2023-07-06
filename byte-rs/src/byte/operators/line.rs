use super::*;

pub struct SplitLineOperator;

impl IsOperator for SplitLineOperator {
	fn precedence(&self) -> Precedence {
		Precedence::SplitLines
	}

	fn predicate(&self, node: &Node) -> bool {
		matches!(node, Node::Break(..))
	}

	fn apply(&self, context: &mut OperatorContext, errors: &mut Errors) {
		let _ = errors;

		let mut to_resolve = Vec::new();
		context.nodes().split_by(
			|n| matches!(n, Node::Break(..)),
			|list| {
				to_resolve.push(list.clone());
				let span = list.span();
				Node::Line(list, at(span))
			},
		);

		for it in to_resolve {
			context.resolve_nodes(&it);
		}
	}
}
