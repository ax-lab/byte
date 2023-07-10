use super::*;

pub struct BindOperator;

impl IsOperator for BindOperator {
	fn predicate(&self, node: &Node) -> bool {
		matches!(node.bit(), Bit::Token(Token::Word(..)))
	}

	fn apply(&self, scope: &Scope, nodes: &mut Vec<Node>, context: &mut OperatorContext) -> Result<bool> {
		let _ = context;
		let mut errors = Errors::new();
		let changed = Nodes::replace(nodes, |node| {
			if let Bit::Token(Token::Word(name)) = node.bit() {
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
		if errors.len() > 0 {
			Err(errors)
		} else {
			Ok(changed)
		}
	}
}
