use super::*;

//====================================================================================================================//
// Node operators
//====================================================================================================================//

/// Evaluation order precedence for [`NodeOperator`].
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum NodePrecedence {
	Highest,
	Brackets,
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
	SplitLines,
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
			NodeOperator::SplitLines => Arc::new(OpSplitLine),
			NodeOperator::Let(decl, eq) => Arc::new(OpDecl(decl.clone(), eq.clone())),
			NodeOperator::Bind => Arc::new(OpBind),
			NodeOperator::Print(symbol) => Arc::new(OpPrint(symbol.clone())),
			NodeOperator::Replace(symbol, node) => Arc::new(ReplaceSymbol(symbol.clone(), node.clone())),
			NodeOperator::ParseExpression(ops) => Arc::new(ops.clone()),
			NodeOperator::Comma(symbol) => Arc::new(CommaOperator(symbol.clone())),
			NodeOperator::Brackets(pairs) => Arc::new(pairs.clone()),
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
