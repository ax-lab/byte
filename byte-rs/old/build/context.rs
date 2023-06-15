use std::{
	collections::HashMap,
	path::{Path, PathBuf},
	sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use super::*;

use crate::lexer::*;

/// The context contains all the state necessary to compile and resolve nodes
/// during compilation and to output the result.
///
/// A [`Context`] is just a handle that can be shared by cloning. To create a
/// new instance, either a new root context can be created, or an existing
/// context can be inherited.
#[derive(Clone)]
pub struct Context {
	data: Arc<RwLock<ContextData>>,
	global: Arc<RwLock<Global>>,
}

impl Context {
	/// Create a new root context without any configuration.
	pub fn new_root(base_path: PathBuf) -> Self {
		Self {
			data: Default::default(),
			global: Arc::new(RwLock::new(Global {
				base_path: base_path.into(),
				..Default::default()
			})),
		}
	}

	/// Inherit a new context from the current one. Any configuration in the
	/// parent context, including future changes, is visible to the inherited
	/// instance.
	pub fn inherit(&self) -> Self {
		let mut data = ContextData::default();
		data.parent = Some(self.data.clone());

		// the scanner is individual for each context and inherited at the
		// moment of the context creation (i.e. further changes to the parent
		// scanner will not be visible)
		data.scanner = self.scanner();

		let data = RwLock::new(data).into();
		Self {
			data,
			global: self.global.clone(),
		}
	}

	/// Returns the parent context, if any.
	pub fn parent(&self) -> Option<Context> {
		let data = self.data();
		data.parent.as_ref().map(|data| Self {
			data: data.clone(),
			global: self.global.clone(),
		})
	}

	/// The root parent context for the current context.
	pub fn root(&self) -> Context {
		self.parent()
			.map(|parent| parent.root())
			.unwrap_or_else(|| self.clone())
	}

	//----------------------------------------------------------------------------------------------------------------//
	// Configuration
	//----------------------------------------------------------------------------------------------------------------//

	/// Name for the current context. Used mostly for debugging and error
	/// messages.
	pub fn name(&self) -> Option<Str> {
		self.get_inherited(|data| data.name.clone())
	}

	/// Base path for any module imports and file loading.
	pub fn base_path(&self) -> Arc<PathBuf> {
		self.global().base_path.clone()
	}

	/// Set the name for the current context. Used mostly for debugging and
	/// error messages.
	pub fn set_name<T: Into<String>>(&mut self, name: T) {
		self.set(|data| data.name = Some(Str::from(name)));
	}

	/// Module that owns this context or any of the parent modules.
	pub fn module(&self) -> Option<Module> {
		self.get_inherited(|data| data.module.clone())
	}

	//----------------------------------------------------------------------------------------------------------------//
	// Error handling
	//----------------------------------------------------------------------------------------------------------------//

	/// True if the context has any error.
	pub fn has_errors(&self) -> bool {
		self.get(|x| x.errors.len() > 0)
	}

	/// Errors for the current context, including errors raised by inherited
	/// contexts.
	pub fn errors(&self) -> Errors {
		self.get(|x| x.errors.clone())
	}

	/// Raise an error for the current context and any parent contexts.
	pub fn raise_error<T: IsValue>(&self, error: T) {
		let error = Value::from(error).with_context(self);
		self.append_error(error);
	}

	/// Raise a list of errors for the current context and any parent contexts.
	pub fn raise_errors(&self, errors: &Errors) {
		let errors = errors.with_context(self);
		self.append_errors(&errors);
	}

	//----------------------------------------------------------------------------------------------------------------//
	// Modules
	//----------------------------------------------------------------------------------------------------------------//

	/// Returns a new instance of the context's scanner.
	pub fn scanner(&self) -> Scanner {
		self.get(|x| x.scanner.clone())
	}

	/// Update the context's scanner.
	pub fn update_scanner(&mut self, scanner: Scanner) {
		self.set(|x| x.scanner = scanner);
	}

	/// Create a new module loading the given input.
	///
	/// The new module will inherit the root context for the current context,
	/// but will not be accessible in any other way.
	pub fn create_module_from_input(&mut self, input: Input) -> Module {
		let mut context = self.root().inherit();
		if let Some(name) = input.name() {
			context.set_name(name);
		}

		let module = Module::new(context.clone(), input);
		context.data_mut().module = Some(module.clone());

		module
	}

	/// Load a module from a file path.
	///
	/// Modules are loaded only once and cached. Loading the same path twice
	/// will return the same module.
	pub fn load_module_from_path<P: AsRef<Path>>(&mut self, path: P) -> Result<Module> {
		let path = path.as_ref();
		let full_path = if path.is_relative() {
			self.base_path().join(path)
		} else {
			path.to_owned()
		};

		// TODO: handle module from a directory

		let modules = {
			let global = self.global.write().unwrap();
			global.modules_by_path.clone()
		};
		let mut modules = modules.write().unwrap();

		let module = std::fs::canonicalize(full_path)
			.and_then(|full_path| {
				if let Some(module) = modules.get(&full_path).cloned() {
					Ok(module)
				} else {
					let input = Input::open(path)?;
					let mut context = self.root().inherit();
					context.set_name(path.to_string_lossy());

					let module = Module::new(context.clone(), input);
					context.data_mut().module = Some(module.clone());

					modules.insert(full_path, module.clone());
					drop(modules);
					Ok(module)
				}
			})
			.map_err(|err| Errors::from(format!("loading `{}`: {err}", path.to_string_lossy())))?;

		Ok(module)
	}

	//----------------------------------------------------------------------------------------------------------------//
	// Node resolution
	//----------------------------------------------------------------------------------------------------------------//

	/// Resolve all pending nodes in the context.
	pub fn resolve(&mut self) {
		let mut resolver = {
			let mut data = self.data.write().unwrap();
			std::mem::take(&mut data.resolver)
		};
		resolver.resolve(self);
	}

	/// Queue nodes to be resolved by the context.
	pub fn queue_nodes<T: IntoIterator<Item = Node>>(&mut self, nodes: T) {
		let mut data = self.data_mut();
		data.resolver.queue_nodes(nodes);
	}

	//----------------------------------------------------------------------------------------------------------------//
	// Internal
	//----------------------------------------------------------------------------------------------------------------//

	fn get<T, P: FnOnce(&ContextData) -> T>(&self, predicate: P) -> T {
		let data = self.data();
		predicate(&data)
	}

	fn set<P: FnOnce(&mut ContextData)>(&mut self, predicate: P) {
		let mut data = self.data_mut();
		predicate(&mut data);
	}

	fn get_inherited<T, P: Fn(&ContextData) -> Option<T>>(&self, predicate: P) -> Option<T> {
		let value = {
			let data = self.data();
			predicate(&data)
		};
		value.or_else(|| self.parent().and_then(|x| x.get_inherited(predicate)))
	}

	fn data(&self) -> RwLockReadGuard<ContextData> {
		match self.data.read() {
			Ok(guard) => guard,
			Err(err) => err.into_inner(), // disregard any poison errors when reading
		}
	}

	fn data_mut(&mut self) -> RwLockWriteGuard<ContextData> {
		self.data.write().unwrap()
	}

	fn global(&self) -> RwLockReadGuard<Global> {
		match self.global.read() {
			Ok(guard) => guard,
			Err(err) => err.into_inner(), // disregard any poison errors when reading
		}
	}

	//----------------------------------------------------------------------------------------------------------------//
	// Internal - errors
	//----------------------------------------------------------------------------------------------------------------//

	fn append_error(&self, error: Value) {
		if let Some(parent) = self.parent() {
			parent.append_error(error.clone());
		}

		let mut data = self.data.write().unwrap();
		data.errors.add(error);
		drop(data);
	}

	fn append_errors(&self, errors: &Errors) {
		if let Some(parent) = self.parent() {
			parent.append_errors(&errors);
		}

		let mut data = self.data.write().unwrap();
		data.errors.append(errors);
		drop(data);
	}
}

//====================================================================================================================//
// Traits
//====================================================================================================================//

impl PartialEq for Context {
	fn eq(&self, other: &Self) -> bool {
		std::ptr::eq(self.data.as_ref(), other.data.as_ref())
	}
}

impl Eq for Context {}

impl std::hash::Hash for Context {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		let data = Arc::as_ptr(&self.data);
		state.write_usize(data as usize);
	}
}

//====================================================================================================================//
// Context data
//====================================================================================================================//

#[derive(Default)]
struct ContextData {
	parent: Option<Arc<RwLock<ContextData>>>,
	scanner: Scanner,
	name: Option<Str>,
	errors: Errors,
	module: Option<Module>,
	resolver: Resolver,
}

#[derive(Default)]
struct Global {
	base_path: Arc<PathBuf>,
	modules_by_path: Arc<RwLock<HashMap<PathBuf, Module>>>,
}

//====================================================================================================================//
// Helpers
//====================================================================================================================//

impl Value {
	pub fn with_context(&self, context: &Context) -> Value {
		if !self.has_field::<FromContext>() {
			self.with_field(FromContext(context.clone()))
		} else {
			self.clone()
		}
	}
}

impl Errors {
	pub fn with_context(&self, context: &Context) -> Errors {
		let errors = self.iter().map(|x| x.with_context(context));
		Errors::from_list(errors)
	}
}

struct FromContext(Context);

has_traits!(FromContext);

impl HasRepr for FromContext {
	fn output_repr(&self, output: &mut Repr) -> std::io::Result<()> {
		write!(output, "FromContext(name={:?})", self.0.name())
	}
}
