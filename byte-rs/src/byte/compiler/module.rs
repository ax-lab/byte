use std::path::Path;

use super::*;

/// Represents an isolated module of code.
#[derive(Clone)]
pub struct Module {
	input: Input,
	has_errors: bool,
}

impl Module {
	pub fn from_path<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
		let input = Input::open(path)?;
		Ok(Self::from_input(input))
	}

	pub fn from_input(input: Input) -> Self {
		Self {
			input,
			has_errors: false,
		}
	}

	pub fn input(&self) -> &Input {
		&self.input
	}

	pub fn has_errors(&self) -> bool {
		self.has_errors
	}

	pub(crate) fn compile_module(&mut self, context: &Context) {
		// First we split the input into broad segments which are then parsed
		// individually.
		let _ = context;
		todo!();

		//--------------------------------------------------------------------//
		// TODO:
		//
		// The below procedure must be applied recursively to the segment
		// contents. In the final implementation, those steps would also be
		// heavily parallelized.
		//--------------------------------------------------------------------//

		//--------------------------------------------------------------------//
		// Step 1 - Lexer configuration
		//--------------------------------------------------------------------//
		//
		// Resolve each segment sequentially looking for lexer directives that
		// affect token parsing. The configuration is stored as a Scanner with
		// each segment inheriting the previous segment's configuration.

		//--------------------------------------------------------------------//
		// Step 2 - Syntax macro resolution and static name binding
		//--------------------------------------------------------------------//
		//
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
