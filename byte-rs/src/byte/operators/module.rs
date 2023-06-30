use super::*;

pub struct ModuleOperator;

impl IsOperator for ModuleOperator {
	fn precedence(&self) -> Precedence {
		Precedence::Modules
	}

	fn predicate(&self, node: &Node) -> bool {
		matches!(node, &Node::Module(..))
	}

	fn apply(&self, context: &mut OperatorContext, errors: &mut Errors) {
		let scope = context.scope();
		let scanner = scope.scanner();
		context.nodes().map_nodes(move |node| {
			if let Node::Module(input) = node.get() {
				let mut cursor = input.start();
				let mut output = Vec::new();
				while let Some(node) = scanner.scan(&mut cursor, errors) {
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