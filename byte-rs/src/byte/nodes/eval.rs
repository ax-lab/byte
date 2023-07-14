use super::*;

//====================================================================================================================//
// Context
//====================================================================================================================//

/// Context for an [`NodeOperator`] application.
pub struct EvalContext {
	nodes: Node,
	scope: Scope,
	new_nodes: Vec<Node>,
	del_nodes: Vec<Node>,
	declares: Vec<(Symbol, Option<usize>, BindingValue)>,
}

impl EvalContext {
	pub fn new(nodes: &Node) -> Self {
		Self {
			nodes: nodes.clone(),
			scope: nodes.scope(),
			new_nodes: Default::default(),
			del_nodes: Default::default(),
			declares: Default::default(),
		}
	}

	pub fn nodes(&self) -> &Node {
		&self.nodes
	}

	pub fn scope(&self) -> &Scope {
		&self.scope
	}

	pub fn scope_handle(&self) -> ScopeHandle {
		self.scope.handle()
	}

	pub fn add_new_node(&mut self, list: &Node) {
		self.new_nodes.push(list.clone());
	}

	pub fn forget_node(&mut self, list: &Node) {
		self.del_nodes.push(list.clone());
	}

	pub fn declare_static(&mut self, symbol: Symbol, value: BindingValue) {
		self.declares.push((symbol, None, value));
	}

	pub fn declare_at(&mut self, symbol: Symbol, offset: usize, value: BindingValue) {
		self.declares.push((symbol, Some(offset), value));
	}

	pub(crate) fn get_nodes_to_add(&mut self, output: &mut Vec<Node>) {
		output.append(&mut self.new_nodes)
	}

	pub(crate) fn get_nodes_to_del(&mut self, output: &mut Vec<Node>) {
		output.append(&mut self.del_nodes)
	}

	pub(crate) fn get_declares(&mut self) -> Vec<(Symbol, Option<usize>, BindingValue)> {
		std::mem::take(&mut self.declares)
	}
}
