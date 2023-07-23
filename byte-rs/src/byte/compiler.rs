use super::*;

/// Main interface for the compiler.
///
/// Provides the default language configuration for a [`Program`] and methods
/// for loading, compiling, and running code.
#[derive(Clone)]
pub struct Compiler {
	data: Arc<CompilerData>,
}

impl Compiler {
	/// Create a new compiler instance with default settings using the current
	/// path as base path.
	pub fn new() -> Self {
		Self {
			data: CompilerData::new().into(),
		}
	}

	//----------------------------------------------------------------------------------------------------------------//
	// Module loading and compilation
	//----------------------------------------------------------------------------------------------------------------//

	pub fn new_matcher(&self) -> Matcher {
		self.data.matcher.as_ref().clone()
	}

	pub fn new_program(&self) -> Program {
		let program = Program::new(self);
		program
	}

	pub fn eval_string<T: AsRef<str>>(&self, input: T) -> Result<Value> {
		let mut program = self.new_program();
		let dump = *self.data.dump.read().unwrap();
		if dump {
			program.enable_dump();
		}
		program.eval("eval", input)
	}

	pub fn enable_dump(&self) {
		let mut dump = self.data.dump.write().unwrap();
		*dump = true;
	}
}

impl Default for Compiler {
	fn default() -> Self {
		Compiler::new()
	}
}

//====================================================================================================================//
// Compiler data
//====================================================================================================================//

static COMMON_SYMBOLS: &[&'static str] = &["(", ")", "[", "]", "{", "}", ";", ":", ",", ".", "=", "!", "?", ".."];
const ALPHA: &'static str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ_abcdefghijklmnopqrstuvwxyz";
const DIGIT: &'static str = "0123456789";

struct CompilerData {
	// default matcher used by any new compiler context
	matcher: Arc<Matcher>,
	dump: Arc<RwLock<bool>>,
}

impl CompilerData {
	pub fn new() -> Self {
		let mut matcher = Matcher::new();
		matcher.register_common_symbols();
		matcher.add_matcher(CommentMatcher);
		matcher.add_matcher(LiteralMatcher);
		matcher.add_matcher(IntegerMatcher);

		CompilerData {
			matcher: matcher.into(),
			dump: Default::default(),
		}
	}
}

impl Compiler {
	pub(crate) fn configure_root_scope(&self, scope: &mut ScopeWriter) {
		configure_default_node_evaluators(scope);
	}
}

impl Matcher {
	pub fn register_common_symbols(&mut self) {
		for it in COMMON_SYMBOLS.iter() {
			self.add_symbol(it);
		}
		self.add_word_chars(ALPHA);
		self.add_word_next_chars(DIGIT);
	}
}
