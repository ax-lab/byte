use super::*;

pub struct PrintOperator;

impl IsOperator for PrintOperator {
	fn can_apply(&self, nodes: &NodeList) -> bool {
		nodes.is_keyword(0, &"print".into())
	}

	fn apply(&self, scope: &Scope, nodes: &mut Vec<Node>, context: &mut OperatorContext) -> Result<bool> {
		let args = nodes[1..].to_vec();
		let args = NodeList::new(scope.handle(), args);
		let print = Bit::Print(args.clone(), "\n").at(context.span());
		*nodes = vec![print];
		context.resolve_nodes(&args);
		Ok(true)
	}
}
