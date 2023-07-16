use super::*;

pub trait ParseFilter {
	fn filter(&self, node: &Node) -> bool;
}

impl Node {
	pub fn can_filter<T: ParseFilter>(&self, op: &T) -> bool {
		self.contains(|x| !op.filter(x))
	}

	pub fn filter<T: ParseFilter>(&mut self, op: &T) {
		self.rewrite(|nodes| {
			*nodes = std::mem::take(nodes).into_iter().filter(|x| op.filter(&x)).collect();
			true
		});
	}
}
