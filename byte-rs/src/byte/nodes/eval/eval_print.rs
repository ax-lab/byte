use super::*;

// TODO: `print` should obviously be a macro, or better yet a "plain" function

pub struct EvalPrint(pub Symbol);

impl IsNodeEval for EvalPrint {
	fn applies(&self, node: &Node) -> bool {
		node.has_keyword(self)
	}

	fn execute(&self, ctx: &mut EvalContext, node: &mut Node) -> Result<()> {
		node.parse_keyword(ctx, self)
	}
}

impl ParseKeyword for EvalPrint {
	fn symbol(&self) -> &Symbol {
		&self.0
	}

	fn new_node(&self, ctx: &mut EvalContext, args: Node, span: Span) -> Result<Node> {
		let _ = ctx;
		Ok(Expr::Print(args, "\n").at(ctx.scope_handle(), span))
	}
}
