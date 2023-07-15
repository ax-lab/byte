use super::*;

pub struct OpBind;

impl ParseReplace for OpBind {
	fn can_replace(&self, node: &Node) -> bool {
		matches!(node.val(), NodeValue::Token(Token::Word(..)))
	}

	fn replace(&self, ctx: &mut OperatorContext, node: &Node) -> Result<Option<Node>> {
		let scope = ctx.scope();
		if let NodeValue::Token(Token::Word(name)) = node.val() {
			let span = node.span().clone();
			if let Some(index) = scope.lookup(&name, Some(node.offset())) {
				let value = NodeValue::Variable(name.clone(), index).at(scope.handle(), span);
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
	fn can_apply(&self, node: &Node) -> bool {
		node.can_replace(self)
	}

	fn eval(&self, ctx: &mut OperatorContext, node: &mut Node) -> Result<()> {
		node.replace(ctx, self)
	}
}
