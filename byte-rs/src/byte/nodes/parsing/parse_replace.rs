use super::*;

pub trait ParseReplace {
	fn can_replace(&self, node: &Node) -> bool;

	fn replace(&self, ctx: &mut OperatorContext, node: &Node) -> Result<Option<Node>>;
}

impl Node {
	pub fn can_replace<T: ParseReplace>(&self, op: &T) -> bool {
		self.contains(|x| op.can_replace(x))
	}

	pub fn replace<T: ParseReplace>(&mut self, ctx: &mut OperatorContext, op: &T) -> Result<()> {
		self.rewrite_res(|nodes| {
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
