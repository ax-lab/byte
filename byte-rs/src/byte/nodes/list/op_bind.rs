use super::*;

pub struct OpBind;

impl ParseReplace for OpBind {
	fn can_replace(&self, node: &Node) -> bool {
		matches!(node.bit(), Bit::Token(Token::Word(..)))
	}

	fn replace(&self, ctx: &mut EvalContext, node: &Node) -> Result<Option<Node>> {
		let scope = ctx.scope();
		if let Bit::Token(Token::Word(name)) = node.bit() {
			let span = node.span().clone();
			if let Some(index) = scope.lookup(name, Some(node.offset())) {
				let value = Bit::Variable(name.clone(), index).at(span);
				Ok(Some(value))
			} else {
				Err(Errors::from(format!("undefined symbol `{name}`"), span))
			}
		} else {
			Ok(None)
		}
	}
}

impl IsNodeOperator for OpBind {
	fn can_apply(&self, nodes: &NodeList) -> bool {
		nodes.can_replace(self)
	}

	fn apply(&self, ctx: &mut EvalContext, nodes: &mut NodeList) -> Result<()> {
		nodes.replace(ctx, self)
	}
}
