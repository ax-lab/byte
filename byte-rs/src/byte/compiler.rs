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
		program.eval("eval", input)
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

static COMMON_SYMBOLS: &[&'static str] = &["(", ")", "[", "]", "{", "}", ";", ":", ",", ".", "=", "!", "?"];
const ALPHA: &'static str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ_abcdefghijklmnopqrstuvwxyz";
const DIGIT: &'static str = "0123456789";

struct CompilerData {
	// default matcher used by any new compiler context
	matcher: Arc<Matcher>,
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
		}
	}
}

impl Compiler {
	pub(crate) fn configure_root_scope(&self, scope: &mut ScopeWriter) {
		// expression parsing
		let ops = default_operators();

		let mut matcher = scope.matcher();
		ops.register_symbols(&mut matcher);
		scope.set_matcher(matcher);

		scope.add_node_operator(NodeOperator::ParseExpression(ops), NodePrecedence::Expression);

		//general parsing
		scope.add_node_operator(NodeOperator::Block(Context::symbol(":")), NodePrecedence::Blocks);
		scope.add_node_operator(NodeOperator::SplitLines, NodePrecedence::SplitLines);
		scope.add_node_operator(NodeOperator::StripComments, NodePrecedence::Comments);
		scope.add_node_operator(
			NodeOperator::Let(Context::symbol("let"), Context::symbol("=")),
			NodePrecedence::Let,
		);
		scope.add_node_operator(NodeOperator::Bind, NodePrecedence::Bind);
		scope.add_node_operator(NodeOperator::Print(Context::symbol("print")), NodePrecedence::Print);
		scope.add_node_operator(NodeOperator::Comma(Context::symbol(",")), NodePrecedence::Comma);

		let ternary = OpTernary(
			Context::symbol("?"),
			Context::symbol(":"),
			Arc::new(|a, b, c, span| Bit::Conditional(a, b, c).at(span)),
		);
		scope.add_node_operator(NodeOperator::Ternary(ternary), NodePrecedence::Ternary);

		// brackets
		let mut brackets = BracketPairs::new();
		brackets.add(
			Context::symbol("("),
			Context::symbol(")"),
			Arc::new(|_, n, _| Bit::Group(n)),
		);

		scope.add_node_operator(NodeOperator::Brackets(brackets), NodePrecedence::Brackets);

		// TODO: handle literal values properly as to not need different precedences

		// boolean
		scope.add_node_operator(
			NodeOperator::Replace(Context::symbol("true"), |span| Bit::Boolean(true).at(span)),
			NodePrecedence::Boolean(true),
		);
		scope.add_node_operator(
			NodeOperator::Replace(Context::symbol("false"), |span| Bit::Boolean(false).at(span)),
			NodePrecedence::Boolean(false),
		);

		// null
		scope.add_node_operator(
			NodeOperator::Replace(Context::symbol("null"), |span| Bit::Null.at(span)),
			NodePrecedence::Null,
		);
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
