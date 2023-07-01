use super::*;

pub struct PrintOperator;

impl IsOperator for PrintOperator {
	fn precedence(&self) -> Precedence {
		Precedence::Print
	}

	fn can_apply(&self, nodes: &NodeList) -> bool {
		nodes.is_keyword(0, "print")
	}

	fn apply(&self, context: &mut OperatorContext, errors: &mut Errors) {
		let _ = errors;
		let nodes = context.nodes();
		let args = nodes.slice(1..);
		let print = Node::Print(args.clone(), "\n").at(nodes.span());
		nodes.replace_all(vec![print]);
		context.resolve_nodes(&args);
	}
}
