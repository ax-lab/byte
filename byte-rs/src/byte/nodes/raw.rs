use super::*;

#[derive(Debug, Eq, PartialEq)]
pub struct RawText(pub Input);

has_traits!(RawText: IsNode);

impl IsNode for RawText {
	fn precedence(&self, context: &Context) -> Option<(Precedence, Sequence)> {
		let _ = context;
		Some((Precedence::RawText, Sequence::AtOnce))
	}

	fn evaluate(&self, context: &mut EvalContext) -> Result<NodeEval> {
		let scanner = context.scanner();
		let Self(input) = self;
		let mut cursor = input.start();
		let mut errors = Errors::new();
		let mut output = Vec::new();
		while let Some(node) = scanner.scan(&mut cursor, &mut errors) {
			output.push(node);
			if !errors.empty() {
				break;
			}
		}

		assert!(cursor.at_end() || !errors.empty());

		context.append_errors(&errors);
		context.replace_current(output);
		Ok(NodeEval::Complete)
	}
}
