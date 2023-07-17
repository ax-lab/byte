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
// Operator Context
//====================================================================================================================//

/// Context for an [`NodeOperator`] application.
pub struct OperatorContext {
	scope: Scope,
	declares: Vec<(Symbol, CodeOffset, Node)>,
}

impl OperatorContext {
	pub fn new(node: &Node) -> Self {
		Self {
			scope: node.scope(),
			declares: Default::default(),
		}
	}

	pub fn scope(&self) -> &Scope {
		&self.scope
	}

	pub fn scope_handle(&self) -> ScopeHandle {
		self.scope.handle()
	}

	pub fn declare(&mut self, symbol: Symbol, offset: CodeOffset, value: Node) {
		self.declares.push((symbol, offset, value));
	}

	pub(crate) fn get_declares(&mut self) -> Vec<(Symbol, CodeOffset, Node)> {
		std::mem::take(&mut self.declares)
	}
}

//====================================================================================================================//
// Node operators
//====================================================================================================================//

/// An operation applicable to a [`Node`] and [`Scope`].
pub trait IsNodeOperator {
	fn applies(&self, node: &Node) -> bool;

	fn execute(&self, ctx: &mut OperatorContext, node: &mut Node) -> Result<()>;
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
	pub fn get_impl(&self) -> Arc<dyn IsNodeOperator> {
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

/*
	NOTE ON PERFORMANCE
	===================

	The algorithm below is absolutely abhorrent and is the core algorithm that
	runs for parsing the input source. Having this be the fastest it can is
	crucial for the compilation performance.

	Ideas to improve performance:

	- Each operator should keep an index of applicable nodes. Finding the next
	  applicable operator and its set of nodes would be O(1).

		- We need to keep a set of active (non-empty) operators sorted by index.
		- The index does not need to be perfect, as that would be very hard to
		  keep. Even with false positives, it should allow quickly culling the
		  set of nodes for an operator and quickly identifying no-ops.
		- Nodes that have been applied an operator, should be saved so that they
		  are out of consideration for that operator.
			- This could be as simple as saving the last processed node ID
			  for each operator (assuming Node IDs are strictly incremental).

	- As nodes are created, we need a fast lookup function to identify which
	  operators could apply to a node. In some cases, it might make more sense
	  to keep an index of specific node types instead.

	- The index above would not be perfect, so operators still need to check
	  their apply condition to nodes. That should be done only for the highest
	  precedence operator globally.

	- We might need a quick way of removing nodes from all indexes (e.g. once
	  they have been absorbed into another node). This could be a culling step
	  in the apply condition.

	- Consider having the tree of nodes as a flattened list of segments. A lot
	  of the nodes would consist of those (e.g. tokens, lines, groups, block
	  segments).

	- Quickly looking up parent info for some of the nodes can be crucial for
	  some of the parsing operators.

	- A lot of the operator can apply locally by simply replacing a node value.
	  This should be a very fast operation.

	Minor:

	- Most scopes won't change the set of operators applicable, so there should
	  be a fast path for that.

*/

pub struct NodeOperation {
	node: Node,
	location: NodeLocation,
	operator: NodeOperator,
	operator_impl: Arc<dyn IsNodeOperator>,
}

impl NodeOperation {
	pub fn node(&self) -> &Node {
		&self.node
	}

	pub fn operator(&self) -> &NodeOperator {
		&self.operator
	}

	pub fn operator_impl(&self) -> &dyn IsNodeOperator {
		&*self.operator_impl
	}

	pub fn parent(&self) -> Option<(&Node, usize)> {
		self.location.parent.as_ref().map(|(node, index)| (node, *index))
	}
}

#[derive(Clone, Default)]
struct NodeLocation {
	parent: Option<(Node, usize)>,
	path: Arc<Vec<(Node, usize)>>,
}

impl NodeLocation {
	pub fn push(&self, parent: Node) -> NodeLocation {
		let mut output = self.clone();
		if let Some((cur_parent, cur_index)) = std::mem::take(&mut output.parent) {
			let path = Arc::make_mut(&mut output.path);
			path.push((cur_parent, cur_index));
		}
		output.parent = Some((parent, 0));
		output
	}

	pub fn set_index(&mut self, index: usize) {
		self.parent = std::mem::take(&mut self.parent).map(|(node, _)| (node, index));
	}
}

impl Node {
	pub fn get_node_operations(
		&self,
		max_precedence: Option<NodePrecedence>,
	) -> Result<Option<(Vec<NodeOperation>, NodePrecedence)>> {
		let location = NodeLocation {
			parent: None,
			path: Vec::new().into(),
		};
		self.do_get_node_operations(max_precedence, &location)
	}

	fn do_get_node_operations(
		&self,
		max_precedence: Option<NodePrecedence>,
		location: &NodeLocation,
	) -> Result<Option<(Vec<NodeOperation>, NodePrecedence)>> {
		let mut errors = Errors::new();

		// collect all operators for child nodes
		let (mut operations, mut max_precedence) = {
			let mut cur_precedence = max_precedence;
			let mut cur_ops = Vec::new();
			let mut location = location.push(self.clone());
			for (index, it) in self.val().children().into_iter().enumerate() {
				location.set_index(index);
				let ops = it.do_get_node_operations(cur_precedence, &location).handle(&mut errors);
				if let Some((mut ops, prec)) = ops {
					if cur_precedence.is_none() || Some(prec) < cur_precedence {
						assert!(ops.len() > 0);
						cur_precedence = Some(prec);
						cur_ops.clear();
						cur_ops.append(&mut ops);
					} else {
						assert!(ops.len() > 0);
						assert!(cur_precedence.is_none() || Some(prec) == cur_precedence);
						cur_ops.append(&mut ops);
					}
				}
			}
			(cur_ops, cur_precedence)
		};

		// collect operators that apply to this node
		let operators = self.scope().get_node_operators().into_iter().filter_map(|(op, prec)| {
			if max_precedence.is_none() || Some(prec) <= max_precedence {
				let op_impl = op.get_impl();
				if op_impl.applies(self) {
					Some((op, op_impl, prec))
				} else {
					None
				}
			} else {
				None
			}
		});

		let mut operators = operators.take_while(|(.., prec)| {
			prec != &NodePrecedence::Never
				&& if let Some(max) = max_precedence {
					prec <= &max
				} else {
					true
				}
		});

		// do we have an operator for this node?
		if let Some((op, op_impl, prec)) = operators.next() {
			// validate that we have only one applicable operator
			let operators = operators.take_while(|(.., op_prec)| op_prec == &prec && prec != NodePrecedence::Never);
			let operators = operators.collect::<Vec<_>>();
			if operators.len() > 0 {
				let mut error = format!("node can accept multiple node operators at the same precedence\n-> {op:?}");
				for (op, ..) in operators {
					let _ = write!(error, ", {op:?}");
				}
				let _ = write!(error.indented(), "\n-> {self:?}");
				errors.add(error, self.span())
			} else {
				// if our operation takes precedence, ignore child operations
				if let Some(max_precedence) = max_precedence {
					if prec < max_precedence {
						operations.clear();
					}
				}

				max_precedence = Some(prec);
				operations.push(NodeOperation {
					node: self.clone(),
					location: location.clone(),
					operator: op,
					operator_impl: op_impl,
				})
			}
		}

		errors.check()?;
		if operations.len() > 0 {
			Ok(Some((operations, max_precedence.unwrap())))
		} else {
			Ok(None)
		}
	}
}
