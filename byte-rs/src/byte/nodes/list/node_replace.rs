use super::*;

pub trait NodeReplace {
	fn can_replace(&self, node: &Node) -> bool;

	fn replace(&self, node: &Node, context: &mut EvalContext) -> Result<Option<Node>>;
}

impl NodeList {
	pub fn can_replace<T: NodeReplace>(&self, op: &T) -> bool {
		self.contains(|x| op.can_replace(x))
	}

	pub fn replace<T: NodeReplace>(&mut self, op: &T, context: &mut EvalContext) -> Result<()> {
		self.write_res(|nodes| {
			let changed = {
				let mut changed = false;
				for it in nodes.iter_mut() {
					if let Some(new_node) = op.replace(&it, context)? {
						*it = new_node;
						changed = true;
					}
				}
				changed
			};
			Ok(changed)
		})
	}
}
