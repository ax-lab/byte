use super::*;

pub trait ParseReplace {
	fn can_replace(&self, node: &Node) -> bool;

	fn replace(&self, ctx: &mut EvalContext, node: &Node) -> Result<Option<Node>>;
}

impl NodeList {
	pub fn can_replace<T: ParseReplace>(&self, op: &T) -> bool {
		self.contains(|x| op.can_replace(x))
	}

	pub fn replace<T: ParseReplace>(&mut self, ctx: &mut EvalContext, op: &T) -> Result<()> {
		self.write_res(|nodes| {
			let changed = {
				let mut changed = false;
				for it in nodes.iter_mut() {
					if let Some(new_node) = op.replace(ctx, &it)? {
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
