use super::*;

pub struct EvalStripComments;

impl ParseFilter for EvalStripComments {
	fn filter(&self, node: &Node) -> bool {
		!matches!(node.token(), Some(Token::Comment))
	}
}

impl IsNodeEval for EvalStripComments {
	fn applies(&self, node: &Node) -> bool {
		node.can_filter(self)
	}

	fn execute(&self, ctx: &mut EvalContext, node: &mut Node) -> Result<()> {
		let _ = ctx;
		node.filter(self);
		Ok(())
	}
}
