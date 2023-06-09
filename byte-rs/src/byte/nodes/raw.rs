use super::*;

#[derive(Debug, Eq, PartialEq)]
pub struct RawText {}

has_traits!(RawText: IsNode);

impl IsNode for RawText {
	fn precedence(&self, context: &Context) -> Option<(Precedence, Sequence)> {
		let _ = context;
		Some((Precedence::RawText, Sequence::SingleStep))
	}

	fn evaluate(&self, context: &mut EvalContext) -> Result<NodeEval> {
		let _ = context;
		// let index = context.current_index();
		// let span = context.current().span().cloned();
		// let source = TextSource::new_at(self.text, span);
		// let nodes = source.load();

		// context.replace_node_at(index, nodes);
		todo!()
	}
}
