use super::*;

pub struct OpPrint(pub Symbol);

impl IsNodeOperator for OpPrint {
	fn applies(&self, node: &Node) -> bool {
		node.has_keyword(self)
	}

	fn execute(&self, ctx: &mut OperatorContext, node: &mut Node) -> Result<()> {
		node.parse_keyword(ctx, self)
	}
}

impl ParseKeyword for OpPrint {
	fn symbol(&self) -> &Symbol {
		&self.0
	}

	fn new_node(&self, ctx: &mut OperatorContext, args: Node, span: Span) -> Result<Node> {
		let _ = ctx;
		Ok(NodeValue::Print(args, "\n").at(ctx.scope_handle(), span))
	}
}
