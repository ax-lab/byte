use super::*;

pub mod op_bind;
pub mod op_brackets;
pub mod op_comma;
pub mod op_decl;
pub mod op_expr;
pub mod op_for;
pub mod op_if;
pub mod op_parse_blocks;
pub mod op_print;
pub mod op_replace_symbol;
pub mod op_split_line;
pub mod op_strip_comments;
pub mod op_ternary;
pub mod op_unraw;

pub use op_bind::*;
pub use op_brackets::*;
pub use op_comma::*;
pub use op_decl::*;
pub use op_expr::*;
pub use op_for::*;
pub use op_if::*;
pub use op_parse_blocks::*;
pub use op_print::*;
pub use op_replace_symbol::*;
pub use op_split_line::*;
pub use op_strip_comments::*;
pub use op_ternary::*;
pub use op_unraw::*;

//====================================================================================================================//
// Node operators
//====================================================================================================================//

/// An operation applicable to a [`Node`] and [`Scope`].
pub trait IsNodeOperator {
	fn eval(&self, ctx: &mut EvalContext, node: &mut Node) -> Result<()>;

	fn can_apply(&self, node: &Node) -> bool;
}

/// Evaluation order precedence for [`NodeOperator`].
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum NodePrecedence {
	Highest,
	Brackets,
	Blocks,
	Comments,
	If, // before `SplitLines` because of if..else
	For,
	SplitLines,
	Const,
	Let,
	Print,
	Comma,
	Ternary,
	Expression,
	Boolean(bool),
	Null,
	Bind,
	Least,
	Never,
}

impl NodePrecedence {
	pub fn off(self) -> Self {
		Self::Never
	}
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum NodeOperator {
	Brackets(BracketPairs),
	Block(Symbol),
	SplitLines,
	StripComments,
	Let(Symbol, Symbol, Decl),
	If(Symbol, Symbol),
	For(Symbol, Symbol, Symbol),
	Ternary(OpTernary),
	Print(Symbol),
	Comma(Symbol),
	Replace(Symbol, fn(ScopeHandle, Span) -> Node),
	Bind,
	ParseExpression(OperatorSet),
	Unraw,
}

impl NodeOperator {
	pub fn get_for_node(&self, node: &Node) -> Option<Arc<dyn IsNodeOperator>> {
		if let NodeValue::Raw(..) = node.val() {
			Some(self.get_impl())
		} else {
			None
		}
	}

	fn get_impl(&self) -> Arc<dyn IsNodeOperator> {
		match self {
			NodeOperator::Brackets(pairs) => Arc::new(pairs.clone()),
			NodeOperator::Block(symbol) => Arc::new(OpParseBlocks(symbol.clone())),
			NodeOperator::SplitLines => Arc::new(OpSplitLine),
			NodeOperator::StripComments => Arc::new(OpStripComments),
			NodeOperator::Let(decl, eq, mode) => Arc::new(OpDecl(decl.clone(), eq.clone(), *mode)),
			NodeOperator::If(s_if, s_else) => Arc::new(OpIf::new(s_if.clone(), s_else.clone())),
			NodeOperator::For(a, b, c) => Arc::new(OpFor(a.clone(), b.clone(), c.clone())),
			NodeOperator::Bind => Arc::new(OpBind),
			NodeOperator::Print(symbol) => Arc::new(OpPrint(symbol.clone())),
			NodeOperator::Replace(symbol, node) => Arc::new(ReplaceSymbol(symbol.clone(), node.clone())),
			NodeOperator::ParseExpression(ops) => Arc::new(ops.clone()),
			NodeOperator::Comma(symbol) => Arc::new(CommaOperator(symbol.clone())),
			NodeOperator::Ternary(op) => Arc::new(op.clone()),
			NodeOperator::Unraw => Arc::new(OpUnraw),
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

pub fn configure_default_node_operators(scope: &mut ScopeWriter) {
	// expression parsing
	let ops = default_operators();

	let mut matcher = scope.matcher();
	ops.register_symbols(&mut matcher);
	scope.set_matcher(matcher);

	scope.add_node_operator(NodeOperator::ParseExpression(ops), NodePrecedence::Expression);
	scope.add_node_operator(NodeOperator::Unraw, NodePrecedence::Least);

	//general parsing
	scope.add_node_operator(NodeOperator::Block(Context::symbol(":")), NodePrecedence::Blocks);
	scope.add_node_operator(NodeOperator::SplitLines, NodePrecedence::SplitLines);
	scope.add_node_operator(NodeOperator::StripComments, NodePrecedence::Comments);
	scope.add_node_operator(
		NodeOperator::Let(Context::symbol("let"), Context::symbol("="), Decl::Let),
		NodePrecedence::Let,
	);
	scope.add_node_operator(
		NodeOperator::Let(Context::symbol("const"), Context::symbol("="), Decl::Const),
		NodePrecedence::Const,
	);
	scope.add_node_operator(
		NodeOperator::If(Context::symbol("if"), Context::symbol("else")),
		NodePrecedence::If,
	);
	scope.add_node_operator(
		NodeOperator::For(Context::symbol("for"), Context::symbol("in"), Context::symbol("..")),
		NodePrecedence::For,
	);
	scope.add_node_operator(NodeOperator::Bind, NodePrecedence::Bind);
	scope.add_node_operator(NodeOperator::Print(Context::symbol("print")), NodePrecedence::Print);
	scope.add_node_operator(NodeOperator::Comma(Context::symbol(",")), NodePrecedence::Comma);

	let ternary = OpTernary(
		Context::symbol("?"),
		Context::symbol(":"),
		Arc::new(|a, b, c, scope, span| NodeValue::Conditional(a, b, c).at(scope, span)),
	);
	scope.add_node_operator(NodeOperator::Ternary(ternary), NodePrecedence::Ternary);

	// brackets
	let mut brackets = BracketPairs::new();
	brackets.add(
		Context::symbol("("),
		Context::symbol(")"),
		Arc::new(|_, n, _| NodeValue::Group(n)),
	);

	scope.add_node_operator(NodeOperator::Brackets(brackets), NodePrecedence::Brackets);

	// TODO: handle literal values properly as to not need different precedences

	// boolean
	scope.add_node_operator(
		NodeOperator::Replace(Context::symbol("true"), |scope, span| {
			NodeValue::Boolean(true).at(scope, span)
		}),
		NodePrecedence::Boolean(true),
	);
	scope.add_node_operator(
		NodeOperator::Replace(Context::symbol("false"), |scope, span| {
			NodeValue::Boolean(false).at(scope, span)
		}),
		NodePrecedence::Boolean(false),
	);

	// null
	scope.add_node_operator(
		NodeOperator::Replace(Context::symbol("null"), |scope, span| NodeValue::Null.at(scope, span)),
		NodePrecedence::Null,
	);
}

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

//====================================================================================================================//
// Operator binding logic
//====================================================================================================================//

impl Node {
	pub fn get_next_node_operator(
		&self,
		max_precedence: Option<NodePrecedence>,
	) -> Result<Option<(NodeOperator, Arc<dyn IsNodeOperator>, NodePrecedence)>> {
		let operators = self.scope().get_node_operators().into_iter().filter_map(|(op, prec)| {
			op.get_for_node(self)
				.and_then(|node_op| if node_op.can_apply(self) { Some(node_op) } else { None })
				.map(|node_op| (op, node_op, prec))
		});
		let mut operators = operators.take_while(|(.., prec)| {
			if let Some(max) = max_precedence {
				prec <= &max && prec != &NodePrecedence::Never
			} else {
				true
			}
		});

		if let Some((op, node_op, prec)) = operators.next() {
			let operators = operators.take_while(|(.., op_prec)| op_prec == &prec && prec != NodePrecedence::Never);
			let operators = operators.collect::<Vec<_>>();
			if operators.len() > 0 {
				let mut error =
					format!("ambiguous node list can accept multiple node operators at the same precedence\n-> {op:?}");
				for (op, ..) in operators {
					let _ = write!(error, ", {op:?}");
				}
				let _ = write!(error.indented(), "\n-> {self:?}");
				Err(Errors::from(error, self.span()))
			} else {
				Ok(Some((op, node_op, prec)))
			}
		} else {
			Ok(None)
		}
	}
}
