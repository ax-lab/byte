use super::*;

#[derive(Debug, Eq, PartialEq)]
pub struct RawText(pub Input);

has_traits!(RawText: IsNode);

impl IsNode for RawText {}

pub struct RawTextOp;

impl NodeOperator for RawTextOp {
	fn evaluate(&self, context: &mut ResolveContext) {
		for (index, node) in context.nodes().clone().iter().enumerate() {
			if let Some(RawText(input)) = node.get::<RawText>() {
				let scanner = context.compiler().scanner();
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
				context.replace_index(index, output);
				if errors.len() > 0 {
					context.errors_mut().append(&errors);
				}
			}
		}
	}
}
