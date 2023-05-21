use std::{
	collections::HashMap,
	path::{Path, PathBuf},
	sync::{Arc, RwLock},
};

use super::lexer::*;
use super::*;

#[derive(Clone)]
pub struct Compiler {
	base_path: PathBuf,
	modules_by_path: Arc<RwLock<HashMap<PathBuf, Module>>>,
	modules: Arc<RwLock<Vec<Module>>>,
	errors: Arc<RwLock<Errors>>,
	tracer: DebugLog,
	scanner: Scanner,
}

impl Compiler {
	pub fn new() -> Self {
		let base_path = std::env::current_dir().expect("failed to get working dir");
		Self {
			base_path,
			modules_by_path: Default::default(),
			modules: Default::default(),
			errors: Default::default(),
			tracer: Default::default(),
			scanner: Default::default(),
		}
	}

	pub fn new_with_defaults() -> Self {
		let mut compiler = Self::new();
		compiler.load_defaults();
		compiler
	}

	//================================================================================================================//
	// Error handling
	//================================================================================================================//

	pub fn errors(&self) -> Errors {
		let mut errors = self.errors.read().unwrap().clone();
		for module in self.modules_by_path.read().unwrap().values() {
			errors.append(&module.errors());
		}
		errors
	}

	pub fn add_error<T: IsValue>(&self, error: T) {
		self.errors.write().unwrap().add(error);
	}

	pub fn append_errors(&self, errors: &Errors) {
		self.errors.write().unwrap().append(errors);
	}

	//----------------------------------------------------------------------------------------------------------------//
	// Setup
	//----------------------------------------------------------------------------------------------------------------//

	pub fn load_defaults(&mut self) {
		use super::lang::*;

		let scanner = &mut self.scanner;

		scanner.add_matcher(IdentifierMatcher);
		scanner.add_matcher(IntegerMatcher);
		scanner.add_matcher(LiteralMatcher);
		scanner.add_matcher(CommentMatcher);

		Op::add_symbols(scanner);

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
		let path = path.as_ref();
		let full_path = if path.is_relative() {
			self.base_path.join(path)
		} else {
			path.to_owned()
		};

		let module = std::fs::canonicalize(full_path)
			.and_then(|full_path| {
				let mut modules = self.modules_by_path.write().unwrap();
				if let Some(module) = modules.get(&full_path).cloned() {
					Ok(module)
				} else {
					let mut module = Module::from_path(&full_path)?;
					modules.insert(full_path, module.clone());
					drop(modules);
					self.load_module(&mut module);
					Ok(module)
				}
			})
			.map_err(|err| Errors::from(format!("loading `{}`: {err}", path.to_string_lossy())))?;

		Ok(module)
	}

	pub fn load_input(&mut self, input: Input) -> Module {
		let mut module = Module::from_input(input);
		self.load_module(&mut module);
		module
	}

	fn load_module(&mut self, module: &mut Module) {
		let mut modules = self.modules.write().unwrap();
		modules.push(module.clone());
		module.load_input_segments(self);
	}

	//----------------------------------------------------------------------------------------------------------------//
	// Resolution
	//----------------------------------------------------------------------------------------------------------------//

	pub fn resolve_all(&mut self) {
		let mut modules = self.modules.write().unwrap();
		let mut pending: Vec<Module> = modules.iter().cloned().collect();
		while pending.len() > 0 {
			let mut changed = false;
			let mut all_changes = Vec::new();
			pending = pending
				.into_iter()
				.filter_map(|mut module| match module.resolve_next(self) {
					ResolveResult::Done => None,
					ResolveResult::Pass => Some(module),
					ResolveResult::Changed(mut changes) => {
						all_changes.append(&mut changes);
						changed = true;
						Some(module)
					}
				})
				.collect();

			if !changed {
				break;
			}
		}

		drop(pending);
		for it in modules.iter_mut() {
			it.compile_code(self);
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
