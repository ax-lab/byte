use std::{
	path::Path,
	sync::{Arc, RwLock},
};

use crate::lexer::*;

use super::*;

/// Represents an isolated module of code.
#[derive(Clone)]
pub struct Module {
	input: Input,
	data: Arc<RwLock<ModuleData>>,
}

#[derive(Default)]
struct ModuleData {
	static_scope: Scope,
	errors: Errors,
	resolver: Resolver,
	scanner: Scanner,
}

impl Module {
	pub fn from_path<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
		let input = Input::open(path)?;
		Ok(Self::from_input(input))
	}

	pub fn from_input(input: Input) -> Self {
		Self {
			input,
			data: Default::default(),
		}
	}

	pub fn input(&self) -> &Input {
		&self.input
	}

	pub fn errors(&self) -> Errors {
		self.data.read().unwrap().errors.clone()
	}

	pub fn has_errors(&self) -> bool {
		self.data.read().unwrap().errors.clone().len() > 0
	}

	pub fn code(&self) -> Code {
		todo!();
	}

	//----------------------------------------------------------------------------------------------------------------//
	// Module compilation & resolution
	//----------------------------------------------------------------------------------------------------------------//

	pub(crate) fn resolve_next(&mut self, compiler: &Compiler) -> ResolveResult {
		let mut data = self.data.write().unwrap();
		data.resolver.step(compiler, self)
	}

	pub(crate) fn compile_code(&mut self, compiler: &Compiler) {
		let _ = compiler;
		todo!()
	}

	pub(crate) fn load_input_segments(&mut self, compiler: &Compiler) {
		let mut data = self.data.write().unwrap();

		//--------------------------------------------------------------------//
		// Step 1 - Parse the input into segments
		//--------------------------------------------------------------------//
		//
		// This includes lexical analysis and parsing the raw segments that
		// will be resolved into the module code.

		let mut scanner = compiler.new_scanner();

		let mut errors = &mut data.errors;
		let tokens = NodeList::tokenize(self.input.clone(), &mut scanner, &mut errors);

		let mut stream = tokens.into_iter();
		let nodes = if errors.empty() {
			parse_segments(&scanner, &mut stream, &mut errors)
		} else {
			Vec::new()
		};

		data.static_scope = compiler.new_scope();

		data.resolver.push(nodes);
		data.scanner = scanner;

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
