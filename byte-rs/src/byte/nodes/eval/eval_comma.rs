use super::*;

pub struct SplitComma(pub Symbol);

impl ParseSplitSequence for SplitComma {
	fn is_split(&self, node: &Node) -> bool {
		if let Expr::Token(Token::Symbol(symbol)) = node.expr() {
			symbol == self.0
		} else {
			false
		}
	}

	fn new_node(&self, ctx: &mut EvalContext, node: Vec<Node>, span: Span) -> Result<Node> {
		let _ = ctx;
		Ok(Expr::Sequence(node.into()).at(ctx.scope_handle(), span))
	}
}

impl IsNodeEval for SplitComma {
	fn applies(&self, node: &Node) -> bool {
		node.can_split_sequence(self)
	}

	fn execute(&self, ctx: &mut EvalContext, node: &mut Node) -> Result<()> {
		node.split_sequence(ctx, self)
	}
}
