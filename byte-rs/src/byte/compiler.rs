use super::*;

/// Main interface for loading, compiling, and running code.
///
/// This is also the parent and ultimate owner of all compilation and runtime
/// data for any given compilation context.
pub struct Compiler {
	data: Arc<CompilerData>,
}

impl Compiler {
	/// Create a new compiler instance with default settings using the current
	/// path as base path.
	pub fn new() -> Self {
		Self::new_with_path(".").unwrap()
	}

	/// Create a new compiler instance with default settings and the given
	/// base path.
	pub fn new_with_path<T: AsRef<Path>>(base_path: T) -> Result<Self> {
		let base_path = std::fs::canonicalize(base_path)?;
		Ok(Self {
			data: CompilerData::new(base_path),
		})
	}

	//----------------------------------------------------------------------------------------------------------------//
	// Module loading and compilation
	//----------------------------------------------------------------------------------------------------------------//

	pub fn scanner(&self) -> &Scanner {
		&self.data.scanner
	}

	pub fn new_context(&self) -> Context {
		let mut context = Context::new(self);
		context.declare_operator(Precedence::RawText, RawTextOp);
		context
	}

	pub fn eval_string<T: AsRef<str>>(&self, input: T) -> Result<Value> {
		let module = self.load_string(input);
		module.eval()
	}

	pub fn load_file<T: AsRef<Path>>(&self, path: T) -> Result<Module> {
		self.do_load_file(path.as_ref()).map_err(|err| {
			let path = path.as_ref().to_string_lossy();
			Errors::from(format!("loading `{path}`: {err}"))
		})
	}

	fn do_load_file<T: AsRef<Path>>(&self, path: T) -> std::io::Result<Module> {
		let path = path.as_ref();
		let full_path = if path.is_relative() {
			self.data.base_path.join(path)
		} else {
			path.to_owned()
		};

		let full_path = std::fs::canonicalize(full_path)?;

		// TODO: handle module from a directory

		let mut modules = { self.data.modules_by_path.write().unwrap() };
		if let Some(module) = modules.get(&full_path) {
			Ok(module.clone())
		} else {
			let input = Input::open(path)?;
			let module = Module::new(self, input);
			modules.insert(full_path, module.clone());
			Ok(module)
		}
	}

	pub fn load_string<T: AsRef<str>>(&self, data: T) -> Module {
		let data = data.as_ref().as_bytes();
		let input = Input::new("string", data.to_vec());
		self.load_input(input)
	}

	pub fn load_input(&self, input: Input) -> Module {
		Module::new(self, input)
	}

	//----------------------------------------------------------------------------------------------------------------//
	// Compiler data
	//----------------------------------------------------------------------------------------------------------------//

	/// Return a weak reference to this compiler instance that can be used to
	/// retrieve the a full [`Compiler`] instance.
	pub fn get_ref(&self) -> CompilerRef {
		let data = Arc::downgrade(&self.data);
		CompilerRef { data }
	}

	/// Store any generic data with the compiler.
	pub fn store<T: Cell>(&self, data: T) -> Handle<T> {
		let data = self.data.arena.get().store(data);
		self.make_handle(data)
	}

	/// Get a reference to the value of a handle. The handle MUST be from this
	/// same compiler instance.
	pub fn get<T: ?Sized>(&self, handle: Handle<T>) -> &T {
		assert!(handle.compiler == self);
		unsafe { &*handle.as_ptr() }
	}

	/// Store a string in the compiler instance, deduplicating values.
	///
	/// Calling this method with the same string value, will always return the
	/// same string reference.
	pub fn intern<T: AsRef<str>>(&self, str: T) -> Handle<str> {
		let str = str.as_ref();
		let names = self.data.strings.read().unwrap();
		if let Some(value) = names.get(str) {
			self.make_handle(value.as_str())
		} else {
			drop(names);
			let mut names = self.data.strings.write().unwrap();
			names.insert(str.to_string());
			let value = names.get(str).unwrap();
			self.make_handle(value.as_str())
		}
	}

	/// Binds the lifetime of the given reference to self.
	///
	/// This should only be used to rebind immutable references to compiler
	/// data that were obtained through a local intermediate (e.g. a mutex
	/// guard on an outer container with read-only items).
	#[inline(always)]
	fn make_handle<'a, T: ?Sized>(&self, data: &'a T) -> Handle<T> {
		let compiler = self.get_ref();
		Handle { compiler, data }
	}
}

impl Default for Compiler {
	fn default() -> Self {
		Compiler::new()
	}
}

//====================================================================================================================//
// CompilerRef and Handle
//====================================================================================================================//

/// Weak reference to a [`Compiler`].
#[derive(Clone)]
pub struct CompilerRef {
	data: Weak<CompilerData>,
}

impl CompilerRef {
	/// Returns a full reference to the compiler. This will panic if the
	/// compiler has been dropped already.
	pub fn get(&self) -> Compiler {
		let data = self.data.upgrade().expect("using disposed compiler reference");
		Compiler { data }
	}
}

impl PartialEq for CompilerRef {
	fn eq(&self, other: &Self) -> bool {
		self.data.as_ptr() == other.data.as_ptr()
	}
}

impl Eq for CompilerRef {}

impl PartialEq<Compiler> for CompilerRef {
	fn eq(&self, other: &Compiler) -> bool {
		self.data.as_ptr() == other.data.as_ref()
	}
}

impl PartialEq<&Compiler> for CompilerRef {
	fn eq(&self, other: &&Compiler) -> bool {
		self.data.as_ptr() == other.data.as_ref()
	}
}

/// Handle to data owned by a [`Compiler`].
pub struct Handle<T: ?Sized> {
	compiler: CompilerRef,
	data: *const T,
}

impl<T: ?Sized> Handle<T> {
	pub fn get(&self) -> HandleRef<T> {
		let compiler = self.compiler.get();
		let data = self.data;
		HandleRef { compiler, data }
	}

	pub fn as_ptr(&self) -> *const T {
		self.data
	}
}

pub struct HandleRef<T: ?Sized> {
	compiler: Compiler,
	data: *const T,
}

impl<T: ?Sized> HandleRef<T> {
	pub fn as_ref(&self) -> &T {
		self
	}
}

impl<T: ?Sized> Clone for Handle<T> {
	fn clone(&self) -> Self {
		Self {
			compiler: self.compiler.clone(),
			data: self.data,
		}
	}
}

impl<T: ?Sized> Deref for HandleRef<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		let _ = &self.compiler;
		unsafe { &*self.data }
	}
}

unsafe impl<T: Send + ?Sized> Send for Handle<T> {}
unsafe impl<T: Sync + ?Sized> Sync for Handle<T> {}

impl<T: PartialEq + ?Sized> PartialEq for Handle<T> {
	fn eq(&self, other: &Self) -> bool {
		if self.data == other.data {
			true
		} else {
			*self.get() == *other.get()
		}
	}
}

impl<T: Eq + ?Sized> Eq for Handle<T> {}

impl<T: PartialEq + ?Sized> PartialEq<T> for Handle<T> {
	fn eq(&self, other: &T) -> bool {
		if self.data == other {
			true
		} else {
			*self.get() == *other
		}
	}
}

impl<T: PartialEq + ?Sized> PartialEq<&T> for Handle<T> {
	fn eq(&self, other: &&T) -> bool {
		if self.data == *other {
			true
		} else {
			*self.get() == **other
		}
	}
}

impl<T: Display + ?Sized> Display for Handle<T> {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		let data = &*self.get();
		write!(f, "{data}")
	}
}

impl<T: Debug + ?Sized> Debug for Handle<T> {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		let data = &*self.get();
		write!(f, "{data:?}")
	}
}

impl<T: Hash + ?Sized> Hash for Handle<T> {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		let data = &*self.get();
		data.hash(state);
	}
}

impl<T: PartialOrd + ?Sized> PartialOrd for Handle<T> {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		let data = &*self.get();
		let other = &*other.get();
		data.partial_cmp(other)
	}
}

impl<T: Ord + ?Sized> Ord for Handle<T> {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		let data = &*self.get();
		let other = &*other.get();
		data.cmp(other)
	}
}

//====================================================================================================================//
// Compiler data
//====================================================================================================================//

struct CompilerData {
	base_path: PathBuf,

	// default scanner used by any new compiler context
	scanner: Scanner,

	// arena storage for global data that is never deallocated
	arena: ArenaSet,

	// storage for interned strings
	strings: Arc<RwLock<HashSet<String>>>,

	// modules loaded from files
	modules_by_path: Arc<RwLock<HashMap<PathBuf, Module>>>,
}

impl CompilerData {
	pub fn new(base_path: PathBuf) -> Arc<Self> {
		Arc::new_cyclic(|data| {
			let compiler = CompilerRef { data: data.clone() };

			let mut scanner = Scanner::new(compiler.clone());
			scanner.register_common_symbols();
			scanner.add_matcher(CommentMatcher);
			scanner.add_matcher(LiteralMatcher);
			scanner.add_matcher(IntegerMatcher);

			CompilerData {
				base_path,
				scanner,
				arena: Default::default(),
				strings: Default::default(),
				modules_by_path: Default::default(),
			}
		})
	}
}

//====================================================================================================================//
// Tests
//====================================================================================================================//

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn intern() {
		let compiler = Compiler::new();
		let a1 = compiler.intern("abc");
		let a2 = compiler.intern("abc");
		let b1 = compiler.intern("123");
		let b2 = compiler.intern("123");

		assert_eq!(a1, "abc");
		assert_eq!(a2, "abc");
		assert_eq!(b1, "123");
		assert_eq!(b2, "123");

		assert!(a1.as_ptr() == a2.as_ptr());
		assert!(b1.as_ptr() == b2.as_ptr());
		assert!(a1.as_ptr() != b1.as_ptr());
	}
}
