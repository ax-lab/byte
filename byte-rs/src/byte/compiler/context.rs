use std::{
	collections::HashMap,
	path::{Path, PathBuf},
	sync::{Arc, Mutex, RwLock},
};

use crate::SegmentParser;

use super::*;

/// Holds the entire context for the compiler.
#[derive(Clone)]
pub struct Context {
	base_path: PathBuf,
	modules: Arc<Mutex<HashMap<PathBuf, Module>>>,
	errors: Arc<RwLock<Errors>>,
	segment_parser: SegmentParser,
}

impl Context {
	pub fn new() -> Self {
		let base_path = std::env::current_dir().expect("failed to get working dir");
		Self {
			base_path,
			modules: Default::default(),
			errors: Default::default(),
			segment_parser: SegmentParser::new(),
		}
	}

	pub fn new_with_defaults() -> Self {
		let mut context = Self::new();
		context.load_defaults();
		context
	}

	pub fn add_error<T: IsValue>(&mut self, error: T) {
		let mut errors = self.errors.write().unwrap();
		errors.add(error);
	}

	pub fn errors(&self) -> Errors {
		self.errors.read().unwrap().clone()
	}

	//----------------------------------------------------------------------------------------------------------------//
	// Setup
	//----------------------------------------------------------------------------------------------------------------//

	pub fn load_defaults(&mut self) {
		self.segment_parser.add_brackets("(", ")");
		self.segment_parser.add_brackets("[", "]");
		self.segment_parser.add_brackets("{", "}");
	}

	pub fn enable_compiler_trace(&mut self) {}

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
				let input = Input::open(&full_path)?;
				let module = Module::from_input(input);
				modules.insert(full_path, module.clone());
				drop(modules);
				self.load_module(&module);
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
		let module = Module::from_input(input);
		self.load_module(&module);
		module
	}

	fn load_module(&mut self, module: &Module) {
		self.add_error("module loading not implemented".at(module.input().span().without_line()));
	}

	//----------------------------------------------------------------------------------------------------------------//
	// Resolution
	//----------------------------------------------------------------------------------------------------------------//

	pub fn wait_resolve(&self) {}
}
