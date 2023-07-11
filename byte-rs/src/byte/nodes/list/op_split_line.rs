use super::*;

pub struct OpSplitLine;

impl NodeListSplit for OpSplitLine {
	fn is_split(&self, node: &Node) -> bool {
		matches!(node.token(), Some(Token::Break))
	}

	fn new_node(&self, nodes: NodeList) -> Result<Node> {
		let span = nodes.span();
		Ok(Bit::Line(nodes).at(span))
	}
}

impl IsNodeOperator for OpSplitLine {
	fn can_apply(&self, nodes: &NodeList) -> bool {
		nodes.can_split(self)
	}

	fn apply(&self, nodes: &mut NodeList, context: &mut EvalContext) -> Result<()> {
		nodes.split(self, context)
	}
}