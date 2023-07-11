use super::*;

pub struct SplitLineOperator;

impl EvalSplitBy for SplitLineOperator {
	fn is_split(&self, node: &Node) -> bool {
		matches!(node.token(), Some(Token::Break))
	}

	fn new_node(&self, scope: &Scope, nodes: Vec<Node>) -> Result<Node> {
		let nodes = NodeList::new(scope.handle(), nodes);
		let span = nodes.span();
		Ok(Bit::Line(nodes).at(span))
	}
}
