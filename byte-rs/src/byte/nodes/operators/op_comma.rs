use super::*;

pub struct CommaOperator(pub Symbol);

impl ParseSplitSequence for CommaOperator {
	fn is_split(&self, node: &Node) -> bool {
		if let NodeValue::Token(Token::Symbol(symbol)) = node.val() {
			symbol == self.0
		} else {
			false
		}
	}

	fn new_node(&self, ctx: &mut OperatorContext, node: Vec<Node>, span: Span) -> Result<Node> {
		let _ = ctx;
		Ok(NodeValue::Sequence(node.into()).at(ctx.scope_handle(), span))
	}
}

impl IsNodeOperator for CommaOperator {
	fn applies(&self, node: &Node) -> bool {
		node.can_split_sequence(self)
	}

	fn execute(&self, ctx: &mut OperatorContext, node: &mut Node) -> Result<()> {
		node.split_sequence(ctx, self)
	}
}
