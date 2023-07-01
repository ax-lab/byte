use super::*;

pub mod bind;
pub mod bracket;
pub mod decl;
pub mod indent;
pub mod line;
pub mod module;
pub mod parse_ops;
pub mod print;

pub use bind::*;
pub use bracket::*;
pub use decl::*;
pub use indent::*;
pub use line::*;
pub use module::*;
pub use parse_ops::*;
pub use print::*;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum Operator {
	Module,
	SplitLines,
	Let,
	Print,
	Bind,
	Binary(ParseBinaryOp),
}

/// Global evaluation precedence for language nodes.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Precedence {
	Highest,
	Module,
	SplitLines,
	Let,
	OpAdditive,
	Print,
	Bind,
	Least,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Grouping {
	Left,
	Right,
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
			Operator::SplitLines => &SplitLineOperator,
			Operator::Let => &LetOperator,
			Operator::Bind => &BindOperator,
			Operator::Print => &PrintOperator,
			Operator::Binary(op) => op,
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
	declares: Vec<(Name, Option<usize>, BindingValue)>,
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
			declares: Default::default(),
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

	pub fn declare_static(&mut self, name: Name, value: BindingValue) {
		self.declares.push((name, None, value));
	}

	pub fn declare_at(&mut self, name: Name, offset: usize, value: BindingValue) {
		self.declares.push((name, Some(offset), value));
	}

	pub(crate) fn get_new_segments(&mut self, output: &mut Vec<NodeList>) {
		output.append(&mut self.new_segments)
	}

	pub(crate) fn get_declares(&mut self) -> Vec<(Name, Option<usize>, BindingValue)> {
		std::mem::take(&mut self.declares)
	}
}
