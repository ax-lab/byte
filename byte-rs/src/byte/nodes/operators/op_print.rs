use super::*;

pub struct OpPrint(pub Symbol);

impl IsNodeOperator for OpPrint {
	fn can_apply(&self, nodes: &NodeList) -> bool {
		nodes.has_keyword(self)
	}

	fn apply(&self, ctx: &mut EvalContext, nodes: &mut NodeList) -> Result<()> {
		nodes.parse_keyword(ctx, self)
	}
}

impl ParseKeyword for OpPrint {
	fn symbol(&self) -> &Symbol {
		&self.0
	}

	fn new_node(&self, ctx: &mut EvalContext, args: NodeList, span: Span) -> Result<Node> {
		let _ = ctx;
		Ok(NodeValue::Print(args, "\n").at(ctx.scope_handle(), span))
	}
}
