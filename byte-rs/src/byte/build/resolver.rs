use std::collections::VecDeque;

use super::*;

use crate::lexer::*;

pub struct ResolveList {
	init: bool,
	nodes: VecDeque<Resolver>,
}

impl ResolveList {
	pub fn new<T: IntoIterator<Item = Node>>(nodes: T) -> Self {
		let nodes = nodes.into_iter().map(Resolver::new).collect();
		Self { init: false, nodes }
	}

	pub fn resolve_step(&mut self, context: &Context, _module: &Module) -> ResolveListResult {
		let mut errors = Errors::new();
		if !self.init {
			self.init = true;
			let mut scanner = context.new_scanner();
			for node in self.nodes.iter_mut() {
				node.init(&mut errors, &mut scanner);
			}

			if errors.empty() {
				ResolveListResult::Continue
			} else {
				ResolveListResult::Error(errors)
			}
		} else {
			todo!()
		}
	}

	pub fn finish(&mut self) -> Errors {
		let mut errors = Errors::new();
		for it in self.nodes.iter_mut() {
			it.finish(&mut errors);
		}
		errors
	}
}

pub enum ResolveListResult {
	Done,
	Pass,
	Continue,
	Error(Errors),
}

/// Resolves a single [`Node`], usually a segment from the input, into a
/// fully resolved AST node, which can then be compiled.
pub struct Resolver {
	node: Node,
	scanner: Scanner,
}

impl Resolver {
	pub fn new(node: Node) -> Self {
		let scanner = Scanner::new();
		Self { node, scanner }
	}

	/// Executes the first step of the node resolution.
	///
	/// This step is executed sequentially for each node, and as such has the
	/// opportunity to modify the state that will be applied to subsequent
	/// nodes.
	pub fn init(&mut self, errors: &mut Errors, scanner: &mut Scanner) {
		if let Err(parse_errors) = self.parse_lexer_directives(scanner) {
			errors.append(parse_errors);
			return;
		}

		self.scanner = scanner.clone();
	}

	/// Execute a step in the node resolution. Node resolution is executed
	/// in parallel between nodes and the results applied transactionally.
	pub fn step(&mut self) -> ResolverResult {
		ResolverResult::Done
	}

	/// Finish resolution for the node. This is a last chance to wrap up
	/// node resolution or generate errors if the node could not be resolved.
	pub fn finish(&mut self, _errors: &mut Errors) {
		println!("finished: {}", self.node);
	}

	fn parse_lexer_directives(&mut self, scanner: &mut Scanner) -> Result<()> {
		// TODO: parse lexer directives and update the scanner
		let _ = scanner;
		Ok(())
	}
}

/// Result of a [`Resolver`] step.
pub enum ResolverResult {
	/// Node is ready to be compiled.
	Done,

	/// Indicates that the node resolution is not finished, but was able to
	/// progress in the current step with no external changes.
	Continue,

	/// Indicates that node resolution is still pending, but cannot be resolved
	/// at the current state.
	Pass,

	/// Apply the given changes to the current scope.
	Apply(Vec<ResolveChange>),

	/// Resolution failed with the given errors.
	Error(Errors),
}

pub enum ResolveChange {
	/// Declare a new name in the static scope of the module.
	Declare { name: String, node: Node },

	/// Export a new name from the module.
	Export { name: String, node: Node },

	/// Request a new module to be imported into the static scope for the
	/// module.
	Import { name: String, path: String },

	/// Remove the current node from resolution and from the final output.
	///
	/// This can be used for nodes like comments, resolution-only nodes with
	/// no output, or other temporary nodes.
	Delete,

	/// Replace the current node with the given list. This provides support
	/// for macro expansion.
	Replace { with: Vec<Node> },

	/// Append the given nodes to the current module.
	Append { nodes: Vec<Node> },

	/// Add an error from the node resolution without aborting the process.
	AddError(Errors),
}
