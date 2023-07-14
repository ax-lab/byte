use super::*;

pub struct OpPrint(pub Symbol);

impl IsNodeOperator for OpPrint {
	fn can_apply(&self, node: &Node) -> bool {
		node.has_keyword(self)
	}

	fn eval(&self, ctx: &mut EvalContext, node: &mut Node) -> Result<()> {
		node.parse_keyword(ctx, self)
	}
}

impl ParseKeyword for OpPrint {
	fn symbol(&self) -> &Symbol {
		&self.0
	}

	fn new_node(&self, ctx: &mut EvalContext, args: Node, span: Span) -> Result<Node> {
		let _ = ctx;
		Ok(NodeValue::Print(args, "\n").at(ctx.scope_handle(), span))
	}
}
