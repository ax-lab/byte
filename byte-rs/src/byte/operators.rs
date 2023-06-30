use super::*;

pub mod bracket;
pub mod indent;
pub mod line;
pub mod module;
pub mod op_binary;
pub mod op_ternary;
pub mod op_unary;

pub use bracket::*;
pub use indent::*;
pub use line::*;
pub use module::*;
pub use op_binary::*;
pub use op_ternary::*;
pub use op_unary::*;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum Operator {
	Module,
	Tokenize,
	SplitLines,
}

/// Global evaluation precedence for language nodes.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Precedence {
	First,
	Modules,
	Lexer,
	LineSplit,
	Last,
}

impl Operator {
	pub fn precedence(&self) -> Precedence {
		self.get_impl().precedence()
	}

	pub fn can_apply(&self, nodes: &NodeList) -> bool {
		self.get_impl().can_apply(nodes)
	}

	pub fn apply(&self, context: &mut OperatorContext, errors: &mut Errors) {
		self.get_impl().apply(context, errors)
	}

	fn get_impl(&self) -> &dyn IsOperator {
		match self {
			Operator::Module => &ModuleOperator,
			Operator::Tokenize => todo!(),
			Operator::SplitLines => &SplitLineOperator,
		}
	}
}

pub trait IsOperator {
	fn precedence(&self) -> Precedence;

	fn apply(&self, context: &mut OperatorContext, errors: &mut Errors);

	fn can_apply(&self, nodes: &NodeList) -> bool {
		nodes.contains(|x| self.predicate(x))
	}

	fn predicate(&self, node: &Node) -> bool {
		let _ = node;
		false
	}
}

pub struct OperatorContext<'a> {
	nodes: &'a mut NodeList,
	program: HandleRef<Program>,
	scope: HandleRef<Scope>,
	version: usize,
	new_segments: Vec<NodeList>,
}

impl<'a> OperatorContext<'a> {
	pub fn new(nodes: &'a mut NodeList) -> Self {
		let scope = nodes.scope();
		let program = scope.program();
		let version = nodes.version();
		Self {
			nodes,
			program,
			scope,
			version,
			new_segments: Default::default(),
		}
	}

	pub fn has_node_changes(&self) -> bool {
		self.nodes.version() > self.version
	}

	pub fn program(&self) -> &Program {
		&self.program
	}

	pub fn scope(&self) -> &Scope {
		&self.scope
	}

	pub fn nodes(&mut self) -> &mut NodeList {
		self.nodes
	}

	pub fn resolve_nodes(&mut self, list: &NodeList) {
		self.new_segments.push(list.clone())
	}

	pub(crate) fn get_new_segments(&mut self, output: &mut Vec<NodeList>) {
		output.append(&mut self.new_segments)
	}
}
