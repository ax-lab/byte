use super::*;

pub struct EvalBind;

impl ParseReplace for EvalBind {
	fn can_replace(&self, node: &Node) -> bool {
		// TODO: symbol tokens can resolve to evaluators with a given precedence that apply to their context.
		matches!(node.expr(), Expr::Token(Token::Word(..)))
	}

	fn replace(&self, ctx: &mut EvalContext, node: &Node) -> Result<Option<Node>> {
		let scope = ctx.scope();
		if let Expr::Token(Token::Word(name)) = node.expr() {
			let span = node.span().clone();
			if let Some(offset) = scope.lookup(&name, &CodeOffset::At(node.offset())) {
				let value = Expr::UnresolvedVariable(name.clone(), offset).at(scope.handle(), span);
				Ok(Some(value))
			} else {
				Err(Errors::from(format!("undefined symbol `{name}`"), span))
			}
		} else {
			Ok(None)
		}
	}
}

impl IsNodeEval for EvalBind {
	fn applies(&self, node: &Node) -> bool {
		node.can_replace(self)
	}

	fn execute(&self, ctx: &mut EvalContext, node: &mut Node) -> Result<()> {
		node.replace(ctx, self)
	}
}
