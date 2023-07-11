use super::*;

pub type TernaryNodeFn = Arc<dyn Fn(NodeList, NodeList, NodeList, Span) -> Node>;

#[derive(Clone)]
pub struct OpTernary(pub Symbol, pub Symbol, pub TernaryNodeFn);

impl IsNodeOperator for OpTernary {
	fn can_apply(&self, nodes: &NodeList) -> bool {
		nodes.has_ternary(self)
	}

	fn apply(&self, nodes: &mut NodeList, ctx: &mut EvalContext) -> Result<()> {
		nodes.parse_ternary(ctx, self)
	}
}

impl NodeTernary for OpTernary {
	fn delimiters(&self) -> (&Symbol, &Symbol) {
		(&self.0, &self.1)
	}

	fn new_node(&self, ctx: &mut EvalContext, a: NodeList, b: NodeList, c: NodeList, span: Span) -> Result<Node> {
		let _ = ctx;
		let node = (self.2)(a, b, c, span);
		Ok(node)
	}
}

impl PartialEq for OpTernary {
	fn eq(&self, other: &Self) -> bool {
		self.0 == other.0 && self.1 == other.1 && Arc::as_ptr(&self.2) == Arc::as_ptr(&other.2)
	}
}

impl Eq for OpTernary {}

impl Hash for OpTernary {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.0.hash(state);
		self.1.hash(state);
	}
}

impl Debug for OpTernary {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		write!(f, "OpTernary({}, {})", self.0, self.1)
	}
}
