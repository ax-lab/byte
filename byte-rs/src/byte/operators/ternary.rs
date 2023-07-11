use super::*;

pub type TernaryNodeFn = Arc<dyn Fn(NodeList, NodeList, NodeList) -> Node>;

#[derive(Clone)]
pub struct TernaryOp(pub Symbol, pub Symbol, pub TernaryNodeFn);

impl Evaluator for TernaryOp {
	fn can_apply(&self, nodes: &NodeList) -> bool {
		nodes.contains_delimiter_pair(&self.0, &self.1)
	}

	fn apply(&self, scope: &Scope, nodes: &mut Vec<Node>, context: &mut EvalContext) -> Result<bool> {
		let (a, b, c) = Nodes::split_ternary(nodes, &self.0, &self.1).unwrap();

		let a = NodeList::new(scope.handle(), a);
		let b = NodeList::new(scope.handle(), b);
		let c = NodeList::new(scope.handle(), c);
		context.resolve_nodes(&a);
		context.resolve_nodes(&b);
		context.resolve_nodes(&c);
		let node = (self.2)(a, b, c);
		*nodes = vec![node];
		Ok(true)
	}
}

impl PartialEq for TernaryOp {
	fn eq(&self, other: &Self) -> bool {
		self.0 == other.0 && self.1 == other.1 && Arc::as_ptr(&self.2) == Arc::as_ptr(&other.2)
	}
}

impl Eq for TernaryOp {}

impl Hash for TernaryOp {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.0.hash(state);
		self.1.hash(state);
	}
}

impl Debug for TernaryOp {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		write!(f, "TernaryOp({}, {})", self.0, self.1)
	}
}
