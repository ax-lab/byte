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

	fn replace(&self, ctx: &mut OperatorContext, node: &Node) -> Result<Option<Node>> {
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

impl IsNodeOperator for ReplaceSymbol {
	fn can_apply(&self, node: &Node) -> bool {
		node.can_replace(self)
	}

	fn eval(&self, ctx: &mut OperatorContext, node: &mut Node) -> Result<()> {
		node.replace(ctx, self)
	}
}
