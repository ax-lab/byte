use super::*;

#[derive(Debug, Eq, PartialEq)]
pub struct Line(pub NodeList);

has_traits!(Line: IsNode);

impl IsNode for Line {}

pub struct SplitLines;

impl NodeOperator for SplitLines {
	fn evaluate(&self, context: &mut ResolveContext) {
		let mut output = Vec::new();
		let mut line = Vec::new();

		let push_line = |line: &mut Vec<NodeValue>, output: &mut Vec<NodeValue>| {
			let line = NodeList::new(std::mem::take(line));
			let line = Line(line);
			output.push(NodeValue::from(line));
		};

		for node in context.nodes().clone().iter() {
			if node.is::<LineBreak>() {
				push_line(&mut line, &mut output);
			} else {
				line.push(node.clone());
			}
		}

		if line.len() > 0 {
			push_line(&mut line, &mut output);
		}

		if output.len() > 1 {
			context.replace_nodes(output);
		}
	}
}
