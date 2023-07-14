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

	fn new_node(&self, ctx: &mut EvalContext, nodes: Vec<NodeList>, span: Span) -> Result<Node> {
		let _ = ctx;
		Ok(NodeValue::Sequence(nodes).at(ctx.scope_handle(), span))
	}
}

impl IsNodeOperator for CommaOperator {
	fn can_apply(&self, nodes: &NodeList) -> bool {
		nodes.can_split_sequence(self)
	}

	fn apply(&self, ctx: &mut EvalContext, nodes: &mut NodeList) -> Result<()> {
		nodes.split_sequence(ctx, self)
	}
}
