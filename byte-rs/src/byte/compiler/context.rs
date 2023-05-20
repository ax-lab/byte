use std::{
	collections::HashMap,
	path::{Path, PathBuf},
	sync::{Arc, Mutex, RwLock},
};

use super::lexer::*;
use super::*;

/// Holds the entire context for the compiler.
#[derive(Clone)]
pub struct Context {
	base_path: PathBuf,
	modules: Arc<Mutex<HashMap<PathBuf, Module>>>,
	errors: Arc<RwLock<Errors>>,
	tracer: DebugLog,
	scanner: Scanner,
}

impl Context {
	pub fn new() -> Self {
		let base_path = std::env::current_dir().expect("failed to get working dir");
		Self {
			base_path,
			modules: Default::default(),
			errors: Default::default(),
			tracer: Default::default(),
			scanner: Scanner::new(),
		}
	}

	pub fn new_with_defaults() -> Self {
		let mut context = Self::new();
		context.load_defaults();
		context
	}

	//================================================================================================================//
	// Error handling
	//================================================================================================================//

	pub fn errors(&self) -> Errors {
		self.errors.read().unwrap().clone()
	}

	pub fn add_error<T: IsValue>(&self, error: T) {
		self.errors.write().unwrap().add(error);
	}

	pub fn append_errors(&self, errors: Errors) {
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

	//----------------------------------------------------------------------------------------------------------------//
	// Modules
	//----------------------------------------------------------------------------------------------------------------//

	pub fn load_file<P: AsRef<Path>>(&mut self, path: P) -> Option<Module> {
		let path = path.as_ref();
		let full_path = if path.is_relative() {
			self.base_path.join(path)
		} else {
			path.to_owned()
		};

		let module = std::fs::canonicalize(full_path).and_then(|full_path| {
			let mut modules = self.modules.lock().unwrap();
			if let Some(module) = modules.get(&full_path).cloned() {
				Ok(module)
			} else {
				let mut module = Module::from_path(&full_path)?;
				modules.insert(full_path, module.clone());
				drop(modules);
				self.load_module(&mut module);
				Ok(module)
			}
		});

		match module {
			Ok(module) => Some(module),
			Err(err) => {
				self.add_error(format!("loading `{}`: {err}", path.to_string_lossy()));
				None
			}
		}
	}

	pub fn load_input(&mut self, input: Input) -> Module {
		let mut module = Module::from_input(input);
		self.load_module(&mut module);
		module
	}

	fn load_module(&mut self, module: &mut Module) {
		module.compile_module(self);
	}

	//----------------------------------------------------------------------------------------------------------------//
	// Resolution
	//----------------------------------------------------------------------------------------------------------------//

	pub fn wait_resolve(&self) {}

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
