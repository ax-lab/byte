use super::*;

#[derive(Debug, Eq, PartialEq)]
pub struct RawText(pub Input);

has_traits!(RawText: IsNode);

impl IsNode for RawText {
	fn precedence(&self) -> Option<(Precedence, Sequence)> {
		Some((Precedence::RawText, Sequence::AtOnce))
	}

	fn evaluate(&self, context: &mut ResolveContext) {
		// TODO: this should be scoped to the node
		let scanner = context.compiler().scanner();
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

		context.replace_self(output);
		if errors.len() > 0 {
			context.errors_mut().append(&errors);
		}
	}
}
