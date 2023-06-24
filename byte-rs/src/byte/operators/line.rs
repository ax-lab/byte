pub use super::*;

pub struct LineOperator;

impl LineOperator {
	pub fn apply(&self, nodes: &NodeValueList, errors: &mut Errors) -> Option<NodeValueList> {
		let _ = errors;
		let output = nodes.split_by(|x| x.is::<LineBreak>());
		let output = output.into_iter().map(|x| NodeValue::from(x));
		let output = NodeValueList::new(output);
		Some(output)
	}
}
