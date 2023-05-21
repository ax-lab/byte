use std::collections::VecDeque;

use super::*;

/// Result of resolving a [`Node`] or [`Module`] step.
///
/// Resolving is the process of expanding or modifying a [`Node`] to the point
/// where is ready for [`Code`] generation.
///
/// Node resolution is incremental and transactional. Each step will make as
/// much progress as it can with the currently available state, and generate
/// a list of changes applying to the next step.
///
/// Once no further progress is being made, a final step forces the nodes to
/// complete their resolution, or generate an error if it can't.
pub enum ResolveResult {
	/// Resolution is complete, without any further step needed until the final
	/// step.
	Done,

	/// Resolution is not complete, but no further progress can be made at the
	/// current state.
	///
	/// This is used by nodes that are waiting for some definition from the
	/// environment.
	Pass,

	/// Indicates that progress has been made, and publishes a list of changes
	/// or requests to the environment.
	Changed(Vec<ResolveChange>),
}

/// Changes resulting from a resolve step.
pub enum ResolveChange {
	/// Declare a new name in the static scope of the module.
	Declare { name: Str, node: Node },

	/// Export a new name from the module.
	Export { name: Str, node: Node },

	/// Request a new module to be imported into the static scope for the
	/// module.
	Import { name: Str, path: Str },

	/// Remove the current node from resolution and from the final output.
	///
	/// This can be used for nodes like comments, resolution-only nodes with
	/// no output, or other temporary nodes.
	RemoveSelf,

	/// Replace the current node with the given list. This provides support
	/// for macro expansion.
	Replace { with: Vec<Node> },

	/// Append the given nodes to the current module.
	Append { nodes: Vec<Node> },
}

/// Provides support for resolving a list of nodes.
#[derive(Default)]
pub struct Resolver {
	list: VecDeque<Node>,
}

impl Resolver {
	pub fn step(&mut self, context: &Context, module: &Module) -> ResolveResult {
		let _ = (context, module);
		todo!()
	}

	pub fn push<T: IntoIterator<Item = Node>>(&mut self, nodes: T) {
		self.list.extend(nodes)
	}

	pub fn finish(self) -> Vec<Node> {
		self.list.into_iter().collect()
	}
}
