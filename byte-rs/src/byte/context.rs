use std::ops::RangeBounds;

use super::*;

/// Provides all context for [`Node`] evaluation, which is at the core of the
/// language parsing and evaluation.
///
/// The context provides methods to evaluate and resolve a list of [`Node`]
/// until they are complete, from which point they can be used to generate
/// executable code.
///
/// It also provides any compiler services that a node might need to complete
/// its resolution (e.g. file loading, module importing, etc.).
///
/// Contexts are designed to be immutable, with context changes being applied
/// in a single transactional step and generating a new context. Additionally,
/// a context can be freely cloned and stored to preserve a given state.
///
/// Contexts can be composed on the fly, which allow for scope rules to be
/// implemented and maintained.
///
/// Nodes can store and update their own contexts internally. This is used,
/// for example, to maintain a node's own internal scope.
#[derive(Clone, Default)]
pub struct Context {}

impl Context {
	pub fn resolve(&self, nodes: NodeSet) -> Result<(NodeSet, Context)> {
		let _ = nodes;
		todo!()
	}
}

/// Wraps a [`Context`] for a [`IsNode::evaluate`] operation.
///
/// The [`EvalContext`] is writable, and tracks changes made to it so they can
/// be applied to the [`Context`] to create the resulting context.
pub struct EvalContext<'a> {
	context: &'a Context,
	errors: Errors,
	nodes: NodeSet,
}

impl<'a> EvalContext<'a> {
	pub fn context(&self) -> &Context {
		self.context
	}

	pub fn current(&self) -> &Node {
		&self.nodes[self.current_index()]
	}

	pub fn current_index(&self) -> usize {
		todo!()
	}

	pub fn nodes(&self) -> &NodeSet {
		&self.nodes
	}

	pub fn errors(&mut self) -> &mut Errors {
		&mut self.errors
	}

	pub fn resolve_bind(&self, name: Name) -> bool {
		let _ = name;
		todo!()
	}

	pub fn get_bind(&self, name: Name) -> Option<Node> {
		let _ = name;
		todo!()
	}

	pub fn require(&self, name: Name, path: &str) {
		let _ = (name, path);
		todo!()
	}

	pub fn replace_node_at(&self, index: usize, nodes: NodeSet) {
		let _ = (index, nodes);
		todo!()
	}

	pub fn replace_nodes<T: RangeBounds<usize>>(&self, range: T, nodes: NodeSet) {
		let _ = (range, nodes);
		todo!()
	}

	pub fn declare(&self, name: Name, node: Node) {
		let _ = (name, node);
		todo!()
	}

	pub fn queue_resolve(&self, context: Context, nodes: NodeSet) -> Option<(NodeSet, Context)> {
		let _ = (context, nodes);
		todo!()
	}
}
