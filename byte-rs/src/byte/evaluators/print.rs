use super::*;

pub struct PrintOperator;

impl IsNodeOperator for PrintOperator {
	fn can_apply(&self, nodes: &NodeList) -> bool {
		nodes.is_keyword(0, &"print".into())
	}

	fn apply(&self, nodes: &mut NodeList, context: &mut EvalContext) -> Result<()> {
		let args = nodes.slice(1..);
		let print = Bit::Print(args.clone(), "\n").at(nodes.span());
		nodes.replace_all(vec![print]);
		context.resolve_nodes(&args);
		Ok(())
	}
}
