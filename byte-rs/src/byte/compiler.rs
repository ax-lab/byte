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

static COMMON_SYMBOLS: &[&'static str] = &[
	"(", ")", "[", "]", "{", "}", ";", ":", ",", ".", "=", "+", "-", "*", "/", "%", "!", "?",
];
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
		//--------------------------------------------------------------------------------------------------------//
		// Operators
		//--------------------------------------------------------------------------------------------------------//

		//general parsing
		scope.add_evaluator(Evaluator::SplitLines(EvalPrecedence::SplitLines));
		scope.add_evaluator(Evaluator::Let(EvalPrecedence::Let));
		scope.add_evaluator(Evaluator::Bind(EvalPrecedence::Bind));
		scope.add_evaluator(Evaluator::Print(EvalPrecedence::Print));
		scope.add_evaluator(Evaluator::Comma(EvalPrecedence::Comma));

		let ternary = TernaryOp(
			Context::symbol("?"),
			Context::symbol(":"),
			Arc::new(|a, b, c| {
				let span = a.span().to(c.span());
				Bit::Conditional(a, b, c).at(span)
			}),
		);
		scope.add_evaluator(Evaluator::Ternary(ternary, EvalPrecedence::Ternary));

		// brackets
		let mut brackets = BracketPairs::new();
		brackets.add(
			Context::symbol("("),
			Context::symbol(")"),
			Arc::new(|_, n, _| Bit::Group(n)),
		);

		scope.add_evaluator(Evaluator::Brackets(brackets, EvalPrecedence::Brackets));

		// boolean
		scope.add_evaluator(Evaluator::Replace(
			Context::symbol("true"),
			|span| Bit::Boolean(true).at(span),
			EvalPrecedence::Boolean(true),
		));
		scope.add_evaluator(Evaluator::Replace(
			Context::symbol("false"),
			|span| Bit::Boolean(false).at(span),
			EvalPrecedence::Boolean(false),
		));

		// null
		scope.add_evaluator(Evaluator::Replace(
			Context::symbol("null"),
			|span| Bit::Null.at(span),
			EvalPrecedence::Null,
		));

		// binary

		let mut ops = OpMap::new();
		ops.add(Context::symbol("="), BinaryOp::Assign);
		scope.add_evaluator(Evaluator::Binary(
			ParseBinaryOp(ops, Grouping::Right),
			EvalPrecedence::OpAssign,
		));

		// additive
		let mut ops = OpMap::new();
		ops.add(Context::symbol("+"), BinaryOp::Add);
		ops.add(Context::symbol("-"), BinaryOp::Sub);
		scope.add_evaluator(Evaluator::Binary(
			ParseBinaryOp(ops, Grouping::Left),
			EvalPrecedence::OpAdditive,
		));

		// multiplicative
		let mut ops = OpMap::new();
		ops.add(Context::symbol("*"), BinaryOp::Mul);
		ops.add(Context::symbol("/"), BinaryOp::Div);
		ops.add(Context::symbol("%"), BinaryOp::Mod);
		scope.add_evaluator(Evaluator::Binary(
			ParseBinaryOp(ops, Grouping::Left),
			EvalPrecedence::OpMultiplicative,
		));

		// boolean
		let mut ops = OpMap::new();
		ops.add(Context::symbol("and"), BinaryOp::And);
		scope.add_evaluator(Evaluator::Binary(
			ParseBinaryOp(ops, Grouping::Right),
			EvalPrecedence::OpBooleanAnd,
		));

		let mut ops = OpMap::new();
		ops.add(Context::symbol("or"), BinaryOp::Or);
		scope.add_evaluator(Evaluator::Binary(
			ParseBinaryOp(ops, Grouping::Right),
			EvalPrecedence::OpBooleanOr,
		));

		// unary

		let mut ops = OpMap::new();
		ops.add(Context::symbol("not"), UnaryOp::Not);
		ops.add(Context::symbol("!"), UnaryOp::Neg);
		ops.add(Context::symbol("+"), UnaryOp::Plus);
		ops.add(Context::symbol("-"), UnaryOp::Minus);
		scope.add_evaluator(Evaluator::UnaryPrefix(
			ParseUnaryPrefixOp(ops),
			EvalPrecedence::OpUnaryPrefix,
		));
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
