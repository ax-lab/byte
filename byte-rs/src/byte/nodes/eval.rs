use super::*;

/// An operation applicable to a [`NodeList`] and [`Scope`].
pub trait IsNodeOperator {
	fn apply(&self, ctx: &mut EvalContext, nodes: &mut NodeList) -> Result<()>;

	fn can_apply(&self, nodes: &NodeList) -> bool;
}

//====================================================================================================================//
// Context
//====================================================================================================================//

/// Context for an [`NodeOperator`] application.
pub struct EvalContext {
	nodes: NodeList,
	scope: Scope,
	new_segments: Vec<NodeList>,
	del_segments: Vec<NodeList>,
	declares: Vec<(Symbol, Option<usize>, BindingValue)>,
}

impl EvalContext {
	pub fn new(nodes: &NodeList) -> Self {
		Self {
			nodes: nodes.clone(),
			scope: nodes.scope(),
			new_segments: Default::default(),
			del_segments: Default::default(),
			declares: Default::default(),
		}
	}

	pub fn nodes(&self) -> &NodeList {
		&self.nodes
	}

	pub fn scope(&self) -> &Scope {
		&self.scope
	}

	pub fn scope_handle(&self) -> ScopeHandle {
		self.scope.handle()
	}

	pub fn add_segment(&mut self, list: &NodeList) {
		if list.len() > 0 {
			self.new_segments.push(list.clone());
		}
	}

	pub fn del_segment(&mut self, list: &NodeList) {
		self.del_segments.push(list.clone());
	}

	pub fn declare_static(&mut self, symbol: Symbol, value: BindingValue) {
		self.declares.push((symbol, None, value));
	}

	pub fn declare_at(&mut self, symbol: Symbol, offset: usize, value: BindingValue) {
		self.declares.push((symbol, Some(offset), value));
	}

	pub(crate) fn get_new_segments(&mut self, output: &mut Vec<NodeList>) {
		output.append(&mut self.new_segments)
	}

	pub(crate) fn get_del_segments(&mut self, output: &mut Vec<NodeList>) {
		output.append(&mut self.del_segments)
	}

	pub(crate) fn get_declares(&mut self) -> Vec<(Symbol, Option<usize>, BindingValue)> {
		std::mem::take(&mut self.declares)
	}
}
