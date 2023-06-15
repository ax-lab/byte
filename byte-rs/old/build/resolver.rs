use super::*;

#[derive(Default)]
pub struct Resolver {
	pending: Vec<Node>,
}

impl Resolver {
	pub fn queue_nodes<T: IntoIterator<Item = Node>>(&mut self, nodes: T) {
		self.pending.extend(nodes)
	}

	pub fn resolve(&mut self, context: &mut Context) {
		/*
			ALGORITHM
			=========

			- LOOP while there are pending nodes in the local context:
				- for each node:
					- call resolve and collect input and output bindings
				- if no bindings are changed, break LOOP
				- apply all output bindings to the scope
				- CHECK for solvable input bindings:
					- collect all known input bindings
					- if there are no bindings left, continue to next LOOP
					- order the bound symbols by "operator precedence"
					- bind the higher precedence group
						- any ambiguous bindings generate an error
					- if any node resolved all its pending input bindings:
						- continue to next LOOP
					- otherwise continue CHECK

			BINDINGS
			========

			Bindings are declared in a scope and bind by name. Inside the
			scope, bindings can also have a lifetime based on the input
			position. This is the case for expression-level bindings, which
			are only defined after the code execution.

			Bindings can be shadowed (e.g. by a let expression), but that
			can also be validated on per-case basis (e.g. to create a sort
			of keyword).

			## Star imports

			Bindings coming from external imports (i.e. star imports) have the
			lowest precedence of all.

			A star import will only bind when there is nothing else. Star
			imports still follow operator precedence in their group.

			Star imports also bind grouped by the reverse order they appear
			in the input (i.e. star imports shadow previous declarations).

			CYCLIC DEFINITIONS
			==================

			File imports are evaluated globally in parallel. If two files have
			a cyclic dependency, their evaluation will eventually stall until
			one of them solves the other dependency.

			If the dependencies are truly cyclical, such that they cannot be
			resolved, then both files will stall and report the dependencies
			as unsolved bindings.

			OPERATORS
			=========

			- Brackets: (), [], {}
				- Evaluate to a group
			- Indents:
				- Operates as a special line break
					- `A v B > C v D < E`
				- Indent and Dedent form a bracketed pair
				- But indents are virtual, so they can be overridden:
					- `A ( > B v C ) D < E` becomes `A ( > B v C < ) D v E`
			- Comments
				- Evaluate to nothing
			- Line breaks
				- Evaluate to a list
		*/

		let mut pending = std::mem::take(&mut self.pending);
		let all_nodes = pending.clone();
		while pending.len() > 0 {
			let mut changed = false;

			let mut all_changes = Vec::new();

			pending = pending
				.into_iter()
				.filter_map(|node| match node.value().resolve(context) {
					ResolveResult::Done => {
						changed = true;
						None
					}
					ResolveResult::Pass => Some(node),
					ResolveResult::Changed(mut changes) => {
						changed = true;
						all_changes.append(&mut changes);
						Some(node)
					}
				})
				.collect();

			for it in all_changes {
				let _ = it;
				todo!()
			}

			if !changed {
				break;
			}

			if context.has_errors() {
				break;
			}
		}

		if !context.has_errors() {
			for node in all_nodes {
				node.value().finalize(context);
			}
		}

		//--------------------------------------------------------------------//
		// Step 1 - Parse the input into segments
		//--------------------------------------------------------------------//
		//
		// This includes lexical analysis and parsing the raw segments that
		// will be resolved into the module code.

		//--------------------------------------------------------------------//
		// Step 2 - Syntax macro resolution and static name binding
		//--------------------------------------------------------------------//
		//
		// The static namespace is visible anywhere in the file, independently
		// of execution order, so it must be resolved first.
		//
		// Only syntax macros can bind symbols to the static namespace, so each
		// segment is matched with available syntax macros that can parse it.
		//
		// Module imports and exports, type definitions, const declarations,
		// static functions, user macros, custom operators: all of these must
		// use a syntax macro so they can be available at expression parsing.

		//--------------------------------------------------------------------//
		// TODO:
		//
		// The below procedure must be applied recursively to the segment
		// contents. In the final implementation, those steps would also be
		// heavily parallelized.
		//--------------------------------------------------------------------//

		// Resolve syntax macro nodes and bind names in the static scope for
		// the current level.
		//
		// Static scope names are available independently of execution order,
		// and as such must be resolved before expression parsing.
		//
		// Nodes don't need to be fully resolved, but any provided scope names
		// must be resolved by the end of this phase, so they are available at
		// the expression parsing phase.
		//
		// Syntax macros are the most powerful constructs, having access to the
		// raw segments before lexical analysis, full control over the static
		// scope, and being able to generate their own raw segments or other
		// nodes.
		//
		// Examples of syntax macros are:
		//
		//   - const declarations
		//   - static functions and types
		//   - module imports and exports
		//   - user macros (syntax and expression)
		//   - custom operators and literals
		//
		// The static scope maintains a list of active syntax macros that can
		// be applied. For each pending segment, matching macros are queried
		// and a single one can be executed (it is an error for more than one
		// macro to match).
		//
		// The syntax macro has access to the static scope, so it can define
		// symbols, including macros, bind exported names, import modules,
		// define operators, etc.
		//
		// The result of a syntax macro is a node. This node may be resolved
		// or could a new segment which will be included in the resolution
		// (this is how macro expansion can be implemented).
		//
		// Note that the resolution is **fully transactional**, with changes
		// being applied in parallel and only visible for the next round of
		// macros.
		//
		// After all syntax macros are resolved, the remaining nodes are then
		// parsed as expressions.
		//
		// At the end of this stage, the static namespace will be fully
		// resolved, including all imported and exported symbols.
		//
		// ## Module importing
		//
		// When importing a module, it's exported scope is linked to the static
		// scope.
		//
		// Imported macros will also be queried when solving a segment, but
		// only after those in the current scope.
		//
		// ## Cyclic dependencies
		//
		// A module can only export names in the static scope. Names provided
		// externally must generally be solvable without waiting on any module
		// dependencies, being provided on a first-pass solve.
		//
		// The exception to the above are "import export" uses, which provide
		// symbols from an external module. Those are particularly susceptible
		// to circular reference problems.
		//
		// To prevent issues, "import export" macros evaluate their names
		// lazily at the end of the macro expansion phase, after the static
		// name binding of all involved modules is complete.

		// TODO: loop through all segments trying to resolve them as syntax
		//       macros. Once all syntax macros are resolved and no new node
		//       identifies as such, proceed to the next phase.
		//
		//       Imports from other modules require the module to be fully
		//       resolved to this stage, so all visible names are known and
		//       imported macros are available.

		//--------------------------------------------------------------------//
		// (3) Syntax macro expansion
		//--------------------------------------------------------------------//
		//
		// Some syntax macros may expand to segments in their own static scope,
		// requiring further analysis. This is an extension of step (2).

		//--------------------------------------------------------------------//
		// (4) Expression parsing
		//--------------------------------------------------------------------//
		//
		// Parse each remaining segment as an expression. The segment is first
		// tokenized and then parsed according to the expression rules.
		//
		// Expression macros can be used to customize parsing. Those can be:
		//
		//   - function-like macros bound to identifiers in the static scope
		//   - macros bound to specific symbols
		//   - generic expression extensions invoked in their respective
		//     context (e.g. values, operators) before other parsing
		//
		// Expressions can be `let` expressions, witch bind names in the active
		// scope. Those are evaluated sequentially, with defined names being
		// available to the expression itself and subsequent expressions.
		//
		// A let expression can also bind to a macro value, in which case it
		// can be used to customize parsing in subsequent expressions.
		//
		// Expressions can also evaluate to block expressions. Those can only
		// appear at the top level, and have then access to their nested block
		// and neighboring expressions.
		//
		// Let and block expressions are always evaluated sequentially.
		//
		// The result of the expression parsing is an expression tree with
		// (generally) untyped nodes. Undeclared identifiers are also left
		// unbound at this stage.

		// TODO: loop through remaining segments parsing them sequentially as
		//       expressions.
		//
		//       For each sequential expression, keep track of bound names,
		//       which default to the static namespace but are overwritten by
		//       `let` expressions.
	}
}

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
