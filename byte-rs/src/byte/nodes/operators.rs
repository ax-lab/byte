use super::*;

pub mod op_bind;
pub mod op_brackets;
pub mod op_comma;
pub mod op_decl;
pub mod op_expr;
pub mod op_parse_blocks;
pub mod op_print;
pub mod op_replace_symbol;
pub mod op_split_line;
pub mod op_strip_comments;
pub mod op_ternary;

pub use op_bind::*;
pub use op_brackets::*;
pub use op_comma::*;
pub use op_decl::*;
pub use op_expr::*;
pub use op_parse_blocks::*;
pub use op_print::*;
pub use op_replace_symbol::*;
pub use op_split_line::*;
pub use op_strip_comments::*;
pub use op_ternary::*;

//====================================================================================================================//
// Node operators
//====================================================================================================================//

/// Evaluation order precedence for [`NodeOperator`].
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum NodePrecedence {
	Highest,
	Brackets,
	Blocks,
	Comments,
	SplitLines,
	Let,
	Print,
	Ternary,
	Comma,
	Expression,
	Boolean(bool),
	Null,
	Bind,
	Least,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum NodeOperator {
	Brackets(BracketPairs),
	Block(Symbol),
	SplitLines,
	StripComments,
	Let(Symbol, Symbol),
	Ternary(OpTernary),
	Print(Symbol),
	Comma(Symbol),
	Replace(Symbol, fn(Span) -> Node),
	Bind,
	ParseExpression(OperatorSet),
}

impl NodeOperator {
	pub fn can_apply(&self, nodes: &NodeList) -> bool {
		self.get_impl().can_apply(nodes)
	}

	pub fn apply(&self, ctx: &mut EvalContext, nodes: &mut NodeList) -> Result<()> {
		self.get_impl().apply(ctx, nodes)
	}

	fn get_impl(&self) -> Arc<dyn IsNodeOperator> {
		match self {
			NodeOperator::Brackets(pairs) => Arc::new(pairs.clone()),
			NodeOperator::Block(symbol) => Arc::new(OpParseBlocks(symbol.clone())),
			NodeOperator::SplitLines => Arc::new(OpSplitLine),
			NodeOperator::StripComments => Arc::new(OpStripComments),
			NodeOperator::Let(decl, eq) => Arc::new(OpDecl(decl.clone(), eq.clone())),
			NodeOperator::Bind => Arc::new(OpBind),
			NodeOperator::Print(symbol) => Arc::new(OpPrint(symbol.clone())),
			NodeOperator::Replace(symbol, node) => Arc::new(ReplaceSymbol(symbol.clone(), node.clone())),
			NodeOperator::ParseExpression(ops) => Arc::new(ops.clone()),
			NodeOperator::Comma(symbol) => Arc::new(CommaOperator(symbol.clone())),
			NodeOperator::Ternary(op) => Arc::new(op.clone()),
		}
	}
}

//====================================================================================================================//
// Expression operators
//====================================================================================================================//

/// Operator precedence for expression parsing.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum OpPrecedence {
	Highest,
	Unary,
	Multiplicative,
	Additive,
	Comparison,
	BooleanAnd,
	BooleanOr,
	BooleanNot,
	Assign,
	Lowest,
}

//====================================================================================================================//
// Standard operators
//====================================================================================================================//

pub fn default_operators() -> OperatorSet {
	let mut ops = OperatorSet::new();

	ops.add(Operator::new_binary(
		"==".into(),
		BinaryOp::CompareEqual,
		OpPrecedence::Comparison,
		Grouping::Left,
	));

	ops.add(Operator::new_binary(
		"=".into(),
		BinaryOp::Assign,
		OpPrecedence::Assign,
		Grouping::Right,
	));

	ops.add(
		Operator::new_binary("+".into(), BinaryOp::Add, OpPrecedence::Additive, Grouping::Left)
			.and_prefix(UnaryOp::Plus, OpPrecedence::Unary),
	);

	ops.add(
		Operator::new_binary("-".into(), BinaryOp::Sub, OpPrecedence::Additive, Grouping::Left)
			.and_prefix(UnaryOp::Minus, OpPrecedence::Unary),
	);

	ops.add(Operator::new_binary(
		"*".into(),
		BinaryOp::Mul,
		OpPrecedence::Multiplicative,
		Grouping::Left,
	));

	ops.add(Operator::new_binary(
		"/".into(),
		BinaryOp::Div,
		OpPrecedence::Multiplicative,
		Grouping::Left,
	));

	ops.add(Operator::new_binary(
		"%".into(),
		BinaryOp::Mod,
		OpPrecedence::Multiplicative,
		Grouping::Left,
	));

	ops.add(Operator::new_binary(
		"and".into(),
		BinaryOp::And,
		OpPrecedence::BooleanAnd,
		Grouping::Right,
	));

	ops.add(Operator::new_binary(
		"or".into(),
		BinaryOp::Or,
		OpPrecedence::BooleanOr,
		Grouping::Right,
	));

	ops.add(Operator::new_prefix(
		"not".into(),
		UnaryOp::Not,
		OpPrecedence::BooleanNot,
	));

	ops.add(Operator::new_prefix("!".into(), UnaryOp::Neg, OpPrecedence::Unary));

	ops
}
