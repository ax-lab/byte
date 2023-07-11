use super::*;

pub struct OpSplitLine;

impl ParseSplitBy for OpSplitLine {
	fn is_split(&self, node: &Node) -> bool {
		matches!(node.token(), Some(Token::Break))
	}

	fn new_node(&self, ctx: &mut EvalContext, nodes: NodeList) -> Result<Node> {
		let _ = ctx;
		let span = nodes.span();
		Ok(Bit::Line(nodes).at(span))
	}
}

impl IsNodeOperator for OpSplitLine {
	fn can_apply(&self, nodes: &NodeList) -> bool {
		nodes.can_split(self)
	}

	fn apply(&self, ctx: &mut EvalContext, nodes: &mut NodeList) -> Result<()> {
		nodes.split(ctx, self)
	}
}
