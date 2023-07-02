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

	pub fn base_path(&self) -> PathBuf {
		self.data.base_path.clone()
	}

	//----------------------------------------------------------------------------------------------------------------//
	// Module loading and compilation
	//----------------------------------------------------------------------------------------------------------------//

	pub fn scanner(&self) -> &Scanner {
		&self.data.scanner
	}

	pub fn new_program(&self) -> Program {
		let program = Program::new(self);
		program
	}

	pub fn eval_string<T: AsRef<str>>(&self, input: T) -> Result<Value> {
		let mut program = self.new_program();
		program.eval("eval", input)
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
	pub fn store<T: Cell>(&self, data: T) -> CompilerHandle<T> {
		let data = self.data.arena.get().store(data);
		self.make_handle(data)
	}

	/// Get a reference to the value of a handle. The handle MUST be from this
	/// same compiler instance.
	pub fn get<T: ?Sized>(&self, handle: CompilerHandle<T>) -> &T {
		assert!(handle.compiler == self);
		unsafe { &*handle.as_ptr() }
	}

	/// Store a string in the compiler instance, deduplicating values.
	///
	/// Calling this method with the same string value, will always return the
	/// same string reference.
	pub fn intern<T: AsRef<str>>(&self, str: T) -> CompilerHandle<str> {
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
	fn make_handle<'a, T: ?Sized>(&self, data: &'a T) -> CompilerHandle<T> {
		let compiler = self.get_ref();
		CompilerHandle { compiler, data }
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
pub struct CompilerHandle<T: ?Sized> {
	compiler: CompilerRef,
	data: *const T,
}

impl<T: ?Sized> CompilerHandle<T> {
	pub fn get(&self) -> CompilerHandleRef<T> {
		let compiler = self.compiler.get();
		let data = self.data;
		CompilerHandleRef { compiler, data }
	}

	pub fn as_ptr(&self) -> *const T {
		self.data
	}
}

pub struct CompilerHandleRef<T: ?Sized> {
	compiler: Compiler,
	data: *const T,
}

impl<T: ?Sized> CompilerHandleRef<T> {
	pub fn as_ref(&self) -> &T {
		self
	}
}

impl<T: ?Sized> Clone for CompilerHandle<T> {
	fn clone(&self) -> Self {
		Self {
			compiler: self.compiler.clone(),
			data: self.data,
		}
	}
}

impl<T: ?Sized> Deref for CompilerHandleRef<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		let _ = &self.compiler;
		unsafe { &*self.data }
	}
}

unsafe impl<T: Send + ?Sized> Send for CompilerHandle<T> {}
unsafe impl<T: Sync + ?Sized> Sync for CompilerHandle<T> {}

impl<T: PartialEq + ?Sized> PartialEq for CompilerHandle<T> {
	fn eq(&self, other: &Self) -> bool {
		if self.data == other.data {
			true
		} else {
			*self.get() == *other.get()
		}
	}
}

impl<T: Eq + ?Sized> Eq for CompilerHandle<T> {}

impl<T: PartialEq + ?Sized> PartialEq<T> for CompilerHandle<T> {
	fn eq(&self, other: &T) -> bool {
		if self.data == other {
			true
		} else {
			*self.get() == *other
		}
	}
}

impl<T: PartialEq + ?Sized> PartialEq<&T> for CompilerHandle<T> {
	fn eq(&self, other: &&T) -> bool {
		if self.data == *other {
			true
		} else {
			*self.get() == **other
		}
	}
}

impl<T: Display + ?Sized> Display for CompilerHandle<T> {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		let data = &*self.get();
		write!(f, "{data}")
	}
}

impl<T: Debug + ?Sized> Debug for CompilerHandle<T> {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		let data = &*self.get();
		write!(f, "{data:?}")
	}
}

impl<T: Hash + ?Sized> Hash for CompilerHandle<T> {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		let data = &*self.get();
		data.hash(state);
	}
}

impl<T: PartialOrd + ?Sized> PartialOrd for CompilerHandle<T> {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		let data = &*self.get();
		let other = &*other.get();
		data.partial_cmp(other)
	}
}

impl<T: Ord + ?Sized> Ord for CompilerHandle<T> {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		let data = &*self.get();
		let other = &*other.get();
		data.cmp(other)
	}
}

//====================================================================================================================//
// Compiler data
//====================================================================================================================//

static COMMON_SYMBOLS: &[&'static str] = &[
	"(", ")", "[", "]", "{", "}", ";", ":", ",", ".", "=", "+", "-", "*", "/", "%", "!", "?",
];
const ALPHA: &'static str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ_abcdefghijklmnopqrstuvwxyz";
const DIGIT: &'static str = "0123456789";

struct CompilerData {
	base_path: PathBuf,

	// default scanner used by any new compiler context
	scanner: Scanner,

	// arena storage for global data that is never deallocated
	arena: ArenaSet,

	// storage for interned strings
	strings: Arc<RwLock<HashSet<String>>>,
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
			}
		})
	}
}

impl Compiler {
	pub(crate) fn configure_root_scope(&self, scope: &mut Scope) {
		//--------------------------------------------------------------------------------------------------------//
		// Operators
		//--------------------------------------------------------------------------------------------------------//

		//general parsing
		scope.add_operator(Operator::Module);
		scope.add_operator(Operator::SplitLines);
		scope.add_operator(Operator::Let);
		scope.add_operator(Operator::Bind);
		scope.add_operator(Operator::Print);
		scope.add_operator(Operator::Comma);

		let ternary = TernaryOp(
			self.get_name("?"),
			self.get_name(":"),
			Arc::new(|a, b, c| Node::Conditional(a, b, c)),
		);
		scope.add_operator(Operator::Ternary(ternary));

		// brackets
		let mut brackets = BracketPairs::new();
		brackets.add(
			self.get_name("("),
			self.get_name(")"),
			Arc::new(|_, n, _| Node::Group(n)),
		);

		scope.add_operator(Operator::Brackets(brackets));

		// boolean
		scope.add_operator(Operator::Replace(
			self.get_name("true"),
			Node::Boolean(true),
			Precedence::Boolean(true),
		));
		scope.add_operator(Operator::Replace(
			self.get_name("false"),
			Node::Boolean(false),
			Precedence::Boolean(false),
		));

		// null
		scope.add_operator(Operator::Replace(self.get_name("null"), Node::Null, Precedence::Null));

		// binary

		// additive
		let mut ops = OpMap::new();
		ops.add(self.get_name("+"), BinaryOp::Add);
		ops.add(self.get_name("-"), BinaryOp::Sub);
		scope.add_operator(Operator::Binary(ParseBinaryOp(
			ops,
			Precedence::OpAdditive,
			Grouping::Left,
		)));

		// multiplicative
		let mut ops = OpMap::new();
		ops.add(self.get_name("*"), BinaryOp::Mul);
		ops.add(self.get_name("/"), BinaryOp::Div);
		ops.add(self.get_name("%"), BinaryOp::Mod);
		scope.add_operator(Operator::Binary(ParseBinaryOp(
			ops,
			Precedence::OpMultiplicative,
			Grouping::Left,
		)));

		// boolean
		let mut ops = OpMap::new();
		ops.add(self.get_name("and"), BinaryOp::And);
		scope.add_operator(Operator::Binary(ParseBinaryOp(
			ops,
			Precedence::OpBooleanAnd,
			Grouping::Right,
		)));

		let mut ops = OpMap::new();
		ops.add(self.get_name("or"), BinaryOp::Or);
		scope.add_operator(Operator::Binary(ParseBinaryOp(
			ops,
			Precedence::OpBooleanOr,
			Grouping::Right,
		)));

		// unary

		let mut ops = OpMap::new();
		ops.add(self.get_name("not"), UnaryOp::Not);
		ops.add(self.get_name("!"), UnaryOp::Neg);
		ops.add(self.get_name("+"), UnaryOp::Plus);
		ops.add(self.get_name("-"), UnaryOp::Minus);
		scope.add_operator(Operator::UnaryPrefix(ParseUnaryPrefixOp(
			ops,
			Precedence::OpUnaryPrefix,
		)));
	}
}

impl Scanner {
	pub fn register_common_symbols(&mut self) {
		for it in COMMON_SYMBOLS.iter() {
			self.add_symbol(it);
		}
		self.add_word_chars(ALPHA);
		self.add_word_next_chars(DIGIT);
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
