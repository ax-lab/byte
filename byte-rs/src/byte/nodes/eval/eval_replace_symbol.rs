use super::*;

pub struct ReplaceSymbol(pub Symbol, pub fn(ScopeHandle, Span) -> Node);

impl ParseReplace for ReplaceSymbol {
	fn can_replace(&self, node: &Node) -> bool {
		if let Some(symbol) = node.symbol() {
			symbol == self.0
		} else {
			false
		}
	}

	fn replace(&self, ctx: &mut EvalContext, node: &Node) -> Result<Option<Node>> {
		let _ = ctx;
		let new_node = &self.1;
		if let Some(symbol) = node.symbol() {
			if symbol == self.0 {
				Ok(Some(new_node(ctx.scope_handle(), node.span())))
			} else {
				Ok(None)
			}
		} else {
			Ok(None)
		}
	}
}

impl IsNodeEval for ReplaceSymbol {
	fn applies(&self, node: &Node) -> bool {
		node.can_replace(self)
	}

	fn execute(&self, ctx: &mut EvalContext, node: &mut Node) -> Result<()> {
		node.replace(ctx, self)
	}
}
