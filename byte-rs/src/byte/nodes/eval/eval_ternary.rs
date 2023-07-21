use super::*;

pub type TernaryNodeFn = Arc<dyn Fn(Node, Node, Node, ScopeHandle, Span) -> Node>;

#[derive(Clone)]
pub struct EvalTernary(pub Symbol, pub Symbol, pub TernaryNodeFn);

impl IsNodeEval for EvalTernary {
	fn applies(&self, node: &Node) -> bool {
		node.has_ternary(self)
	}

	fn execute(&self, ctx: &mut EvalContext, node: &mut Node) -> Result<()> {
		node.parse_ternary(ctx, self)
	}
}

impl ParseTernary for EvalTernary {
	fn delimiters(&self) -> (&Symbol, &Symbol) {
		(&self.0, &self.1)
	}

	fn new_node(&self, ctx: &mut EvalContext, a: Node, b: Node, c: Node, span: Span) -> Result<Node> {
		let _ = ctx;
		let node = (self.2)(a, b, c, ctx.scope_handle(), span);
		Ok(node)
	}
}

impl PartialEq for EvalTernary {
	fn eq(&self, other: &Self) -> bool {
		self.0 == other.0 && self.1 == other.1 && Arc::as_ptr(&self.2) == Arc::as_ptr(&other.2)
	}
}

impl Eq for EvalTernary {}

impl Hash for EvalTernary {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.0.hash(state);
		self.1.hash(state);
	}
}

impl Debug for EvalTernary {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		write!(f, "OpTernary({}, {})", self.0, self.1)
	}
}
