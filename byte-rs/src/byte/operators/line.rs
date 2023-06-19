pub use super::*;

pub struct LineOperator;

impl LineOperator {
	pub fn apply(&self, nodes: &NodeList, errors: &mut Errors) -> Option<NodeList> {
		let _ = errors;
		let output = nodes.split_by(|x| x.is::<LineBreak>());
		let output = output.into_iter().map(|x| {
			let span = x.span();
			Node::from(x, span)
		});
		let output = NodeList::new(output);
		Some(output)
	}
}
