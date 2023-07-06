use super::*;

pub type TernaryNodeFn = Arc<dyn Fn(NodeList, NodeList, NodeList) -> Node>;

#[derive(Clone)]
pub struct TernaryOp(pub Symbol, pub Symbol, pub TernaryNodeFn);

impl IsOperator for TernaryOp {
	fn precedence(&self) -> Precedence {
		Precedence::Ternary
	}

	fn can_apply(&self, nodes: &NodeList) -> bool {
		nodes.contains_delimiter_pair(&self.0, &self.1)
	}

	fn apply(&self, context: &mut OperatorContext, errors: &mut Errors) {
		let _ = errors;
		let mut nodes = context.nodes().clone();
		let (a, b, c) = nodes.split_ternary(&self.0, &self.1).unwrap();

		let a = NodeList::new(nodes.scope_handle(), a);
		let b = NodeList::new(nodes.scope_handle(), b);
		let c = NodeList::new(nodes.scope_handle(), c);
		let span = a.span().clone();
		context.resolve_nodes(&a);
		context.resolve_nodes(&b);
		context.resolve_nodes(&c);
		let node = (self.2)(a, b, c).at(span);
		nodes.replace_all(vec![node]);
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
