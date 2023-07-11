use super::*;

pub struct CommaOperator(pub Symbol);

impl ParseSplitSequence for CommaOperator {
	fn is_split(&self, node: &Node) -> bool {
		if let Bit::Token(Token::Symbol(symbol)) = node.bit() {
			symbol == &self.0
		} else {
			false
		}
	}

	fn new_node(&self, ctx: &mut EvalContext, nodes: Vec<NodeList>, span: Span) -> Result<Node> {
		let _ = ctx;
		Ok(Bit::Sequence(nodes).at(span))
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
