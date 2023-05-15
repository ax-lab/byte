use super::*;

use crate::lexer::{Scanner, TokenStream};

/// Maintains all the state that is locally available for a [`Node`] during
/// compilation.
///
/// The scope is read-only. Nodes that affect the scope must publish their
/// changes during resolution, and those will be applied to the scope for
/// the next step.
pub struct Scope {}

impl Scope {
	/// The parent module for this scope.
	pub fn module(&self) -> Module {
		todo!()
	}

	/// The immediate parent node for the current scope, if available.
	pub fn parent_node(&self) -> Option<Node> {
		todo!()
	}

	/// Next node in the scope, if available.
	pub fn next_node(&self) -> Option<Node> {
		todo!()
	}

	/// Previous node in the scope, if available.
	pub fn prev_node(&self) -> Option<Node> {
		todo!()
	}

	/// Current scanner configuration for lexical analysis.
	pub fn scanner(&self) -> Scanner {
		todo!()
	}

	/// List of errors at the current scope level.
	pub fn errors(&self) -> Errors {
		todo!()
	}

	/// Raise a single error at the current scope level.
	pub fn raise_error<T: IsValue>(&self, error: T) {
		let _ = error;
		todo!()
	}

	/// Raise a list of errors at the current scope level.
	pub fn raise_errors(&self, errors: Errors) {
		let _ = errors;
		todo!()
	}

	/// The static namespace contains symbols for the current scope that
	/// are defined independently of any execution flow.
	///
	/// Types, constants, macros, and static functions are examples of symbols
	/// in this scope. Those are defined anywhere in the scope and aren't
	/// affected by code execution, being defined before execution begins.
	///
	/// This symbols cannot be bound by expressions, but can be shadowed
	/// at the local namespace level.
	pub fn static_namespace(&self) -> Namespace {
		todo!()
	}

	/// Contains symbols exported by the current scope.
	///
	/// This is like the static namespace, with the same rules, but for symbols
	/// that are to be made available outside the scope.
	pub fn export_namespace(&self) -> Namespace {
		todo!()
	}

	/// Contains symbols imported into the current scope.
	///
	/// Symbols are imported as groups. Each group combines one or more export
	/// namespaces.
	///
	/// Symbols are looked up starting from the last import group (i.e., an
	/// import group overrides previous imports).
	///
	/// In a single group, a symbol must be uniquely defined (i.e., ambiguous
	/// symbols are an error).
	pub fn import_namespace(&self) -> Namespace {
		todo!()
	}

	/// The local namespace provides symbols tied to the code execution.
	///
	/// During expression resolution, symbols defined in this namespace are
	/// available only to subsequent nodes in the same scope.
	///
	/// For macro expansion, this can be used to share contextual information
	/// between sequential nodes that form a logical group (e.g. `if else`).
	pub fn local_namespace(&self) -> Namespace {
		todo!()
	}
}

/// Gives access to symbols defined in a [`Scope`].
pub struct Namespace {}

impl Namespace {
	pub fn get_symbol(&self, symbol: &str) -> Option<Node> {
		let _ = symbol;
		todo!()
	}

	/// List syntax macros that support the given node.
	pub fn list_syntax_macros_for(&self, node: &Node) -> Vec<Node> {
		let _ = node;
		todo!()
	}

	/// List value macros that can apply at the given expression position.
	pub fn list_value_macros_for(&self, stream: &TokenStream) -> Vec<Node> {
		let _ = stream;
		todo!()
	}

	/// List operator macros that can apply at the given expression position.
	pub fn list_op_macros_for(&self, stream: &TokenStream) -> Vec<Node> {
		let _ = stream;
		todo!()
	}
}
