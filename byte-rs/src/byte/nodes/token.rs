use super::*;

/// Basic token nodes generated by a default [`Scanner`].
#[derive(Debug, Eq, PartialEq)]
pub enum Token {
	Break,
	Indent(usize),
	Word(String),
	Symbol(String),
}

has_traits!(Token: IsNode);

impl IsNode for Token {
	fn precedence(&self, context: &Context) -> Option<(Precedence, Sequence)> {
		let _ = context;
		todo!()
	}

	fn evaluate(&self, context: &mut EvalContext) -> Result<NodeEval> {
		let _ = context;
		todo!()
	}
}
