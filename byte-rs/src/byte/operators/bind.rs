use super::*;

pub struct BindOperator;

impl IsOperator for BindOperator {
	fn precedence(&self) -> Precedence {
		Precedence::Bind
	}

	fn predicate(&self, node: &Node) -> bool {
		matches!(node.bit(), Bit::Word(..))
	}

	fn apply(&self, context: &mut OperatorContext, errors: &mut Errors) {
		let mut nodes = context.nodes().clone();
		let scope = nodes.scope();
		nodes.replace(|node| {
			if let Bit::Word(name) = node.bit() {
				let span = node.span().clone();
				if let Some(index) = scope.lookup(name, Some(node.offset())) {
					let value = Bit::Variable(name.clone(), index).at(span);
					Some(value)
				} else {
					let error = format!("undefined symbol `{name}`");
					errors.add(error, span);
					None
				}
			} else {
				None
			}
		});
	}
}
