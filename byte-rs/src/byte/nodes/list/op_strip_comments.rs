use super::*;

pub struct OpStripComments;

impl ParseFilter for OpStripComments {
	fn filter(&self, node: &Node) -> bool {
		!matches!(node.token(), Some(Token::Comment))
	}
}

impl IsNodeOperator for OpStripComments {
	fn apply(&self, ctx: &mut EvalContext, nodes: &mut NodeList) -> Result<()> {
		let _ = ctx;
		nodes.filter(self);
		Ok(())
	}

	fn can_apply(&self, nodes: &NodeList) -> bool {
		nodes.can_filter(self)
	}
}
