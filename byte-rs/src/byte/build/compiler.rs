use std::{
	collections::HashSet,
	path::Path,
	sync::{Arc, RwLock},
};

use super::lexer::*;
use super::*;

#[derive(Clone)]
pub struct Compiler {
	context: Context,
	modules: Arc<RwLock<HashSet<Context>>>,
	tracer: DebugLog,
	scanner: Scanner,
}

impl Compiler {
	pub fn new() -> Self {
		let base_path = std::env::current_dir().expect("failed to get working dir");
		let context = Context::new_root(base_path);
		Self {
			context,
			modules: Default::default(),
			tracer: Default::default(),
			scanner: Default::default(),
		}
	}

	pub fn new_with_defaults() -> Self {
		let mut compiler = Self::new();
		compiler.load_defaults();
		compiler
	}

	pub fn has_errors(&self) -> bool {
		self.context.has_errors()
	}

	pub fn errors(&self) -> Errors {
		self.context.errors()
	}

	//----------------------------------------------------------------------------------------------------------------//
	// Setup
	//----------------------------------------------------------------------------------------------------------------//

	pub fn load_defaults(&mut self) {
		use super::lang::*;

		let mut scanner = self.context.scanner();

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

		self.context.update_scanner(scanner);
	}

	pub fn enable_trace_blocks(&mut self) {
		self.tracer.show_blocks();
	}

	pub fn new_scanner(&self) -> Scanner {
		self.scanner.clone()
	}

	pub fn new_scope(&self) -> Scope {
		Scope::default()
	}

	//----------------------------------------------------------------------------------------------------------------//
	// Modules
	//----------------------------------------------------------------------------------------------------------------//

	pub fn load_file<P: AsRef<Path>>(&mut self, path: P) -> Result<Module> {
		let module = self.context.load_module(path)?;

		let mut modules = self.modules.write().unwrap();
		modules.insert(module.context().clone());

		Ok(module)
	}

	pub fn load_input(&mut self, input: Input) -> Module {
		let module = Module::new(self.context.clone(), input);

		let mut modules = self.modules.write().unwrap();
		modules.insert(module.context().clone());

		module
	}

	//----------------------------------------------------------------------------------------------------------------//
	// Resolution
	//----------------------------------------------------------------------------------------------------------------//

	pub fn resolve_all(&mut self) {
		let mut pending: HashSet<Context> = self.modules.read().unwrap().clone();

		while pending.len() > 0 {
			let mut changed = false;
			let mut all_changes = Vec::new();

			let next: Vec<Context> = pending.iter().cloned().collect();
			for it in next.into_iter() {
				match it.resolve_next() {
					ResolveResult::Done => {
						pending.remove(&it);
					}
					ResolveResult::Pass => todo!(),
					ResolveResult::Changed(mut changes) => {
						all_changes.append(&mut changes);
						changed = true;
					}
				};
			}

			if !changed {
				break;
			}
		}

		for it in pending.into_iter() {
			it.resolve_all();
		}

		if !self.has_errors() {
			let modules: HashSet<Context> = self.modules.read().unwrap().clone();
			for it in modules.into_iter() {
				let it: Context = it;
				it.module().clone().map(|mut x| x.compile_code());
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
