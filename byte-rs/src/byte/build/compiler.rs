use std::{
	collections::HashSet,
	path::Path,
	sync::{Arc, RwLock},
};

use super::lexer::*;
use super::*;

#[derive(Clone)]
pub struct Compiler {
	root_context: Context,
	loaded_contexts: Arc<RwLock<HashSet<Context>>>,
	tracer: DebugLog,
}

impl Compiler {
	pub fn new() -> Self {
		let base_path = std::env::current_dir().expect("failed to get working dir");
		let context = Context::new_root(base_path);
		Self {
			root_context: context,
			loaded_contexts: Default::default(),
			tracer: Default::default(),
		}
	}

	pub fn new_with_defaults() -> Self {
		let mut compiler = Self::new();
		compiler.load_defaults();
		compiler
	}

	pub fn has_errors(&self) -> bool {
		self.root_context.has_errors()
	}

	pub fn errors(&self) -> Errors {
		self.root_context.errors()
	}

	//----------------------------------------------------------------------------------------------------------------//
	// Setup
	//----------------------------------------------------------------------------------------------------------------//

	pub fn load_defaults(&mut self) {
		use super::lang::*;

		let mut scanner = self.root_context.scanner();

		scanner.add_matcher(IdentifierMatcher);
		scanner.add_matcher(IntegerMatcher);
		scanner.add_matcher(LiteralMatcher);
		scanner.add_matcher(CommentMatcher);

		Op::add_symbols(&mut scanner);

		scanner.add_bracket_pair("(", ")");
		scanner.add_symbol("(", Token::Symbol("("));
		scanner.add_symbol(")", Token::Symbol(")"));

		scanner.add_bracket_pair("[", "]");
		scanner.add_symbol("[", Token::Symbol("["));
		scanner.add_symbol("]", Token::Symbol("]"));

		scanner.add_bracket_pair("{", "}");
		scanner.add_symbol("{", Token::Symbol("{"));
		scanner.add_symbol("}", Token::Symbol("}"));

		scanner.add_symbol(",", Token::Symbol(","));
		scanner.add_symbol(":", Token::Symbol(":"));
		scanner.add_symbol(";", Token::Symbol(";"));

		self.root_context.update_scanner(scanner);
	}

	pub fn enable_trace_blocks(&mut self) {
		self.tracer.show_blocks();
	}

	//----------------------------------------------------------------------------------------------------------------//
	// Modules
	//----------------------------------------------------------------------------------------------------------------//

	pub fn load_file<P: AsRef<Path>>(&mut self, path: P) -> Result<Module> {
		let module = self.root_context.load_module_from_path(path)?;

		let mut contexts = self.loaded_contexts.write().unwrap();
		contexts.insert(module.context().clone());

		Ok(module)
	}

	pub fn load_input(&mut self, input: Input) -> Module {
		let module = self.root_context.create_module_from_input(input);

		let mut contexts = self.loaded_contexts.write().unwrap();
		contexts.insert(module.context().clone());

		module
	}

	//----------------------------------------------------------------------------------------------------------------//
	// Resolution
	//----------------------------------------------------------------------------------------------------------------//

	pub fn resolve_all(&mut self) {
		let context_list = self.loaded_contexts.write().unwrap();
		let mut context_list: Vec<Context> = context_list.iter().cloned().collect();

		// Contexts are able to coordinate and resolve themselves, so just
		// trigger every module's context resolution
		for context in context_list.iter_mut() {
			context.resolve();
		}

		// trigger the code generation for all modules
		if !self.has_errors() {
			for it in context_list.iter() {
				if let Some(module) = it.module() {
					module.compile_code();
				}
			}
		}
	}

	//================================================================================================================//
	// Trace events
	//================================================================================================================//

	pub fn trace_segments(&self, module: &Module, nodes: &Vec<Node>) {
		self.tracer.dump_blocks(module, nodes);
	}
}

//====================================================================================================================//
// Debugging
//====================================================================================================================//

#[derive(Clone, Default)]
struct DebugLog {
	show_blocks: bool,
}

impl DebugLog {
	pub fn show_blocks(&mut self) {
		self.show_blocks = true;
	}

	pub fn dump_blocks(&self, module: &Module, nodes: &Vec<Node>) {
		if self.show_blocks {
			let mut output = std::io::stdout().lock();
			let _ = write!(output, "\nInput {}:\n\n", module.input());
			Repr::dump_list(&mut output, nodes.iter().cloned());
			let _ = write!(output, "\n");
		}
	}
}
