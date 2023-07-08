use super::*;

pub struct ModuleOperator;

impl IsOperator for ModuleOperator {
	fn precedence(&self) -> Precedence {
		Precedence::Module
	}

	fn predicate(&self, node: &Node) -> bool {
		matches!(node.bit(), Bit::Module(..))
	}

	fn apply(&self, context: &mut OperatorContext, errors: &mut Errors) {
		let scope = context.scope();
		let matcher = scope.matcher();
		context.nodes().map_nodes(move |node| {
			if let Bit::Module(input) = node.bit() {
				let mut cursor = input.clone();
				let mut output = Vec::new();
				while let Some(node) = matcher.scan(&mut cursor, errors) {
					output.push(node);
					if !errors.empty() {
						break;
					}
				}
				assert!(cursor.at_end() || !errors.empty());
				Some(output)
			} else {
				None
			}
		})
	}
}
