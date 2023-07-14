use super::*;

pub struct OpStripComments;

impl ParseFilter for OpStripComments {
	fn filter(&self, node: &Node) -> bool {
		!matches!(node.token(), Some(Token::Comment))
	}
}

impl IsNodeOperator for OpStripComments {
	fn can_apply(&self, node: &Node) -> bool {
		node.can_filter(self)
	}

	fn eval(&self, ctx: &mut EvalContext, node: &mut Node) -> Result<()> {
		let _ = ctx;
		node.filter(self);
		Ok(())
	}
}
