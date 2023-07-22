use super::*;

pub mod eval_bind;
pub mod eval_brackets;
pub mod eval_comma;
pub mod eval_decl;
pub mod eval_for;
pub mod eval_if;
pub mod eval_ops;
pub mod eval_parse_blocks;
pub mod eval_print;
pub mod eval_replace_symbol;
pub mod eval_split_line;
pub mod eval_strip_comments;
pub mod eval_ternary;
pub mod eval_unraw;

pub use eval_bind::*;
pub use eval_brackets::*;
pub use eval_comma::*;
pub use eval_decl::*;
pub use eval_for::*;
pub use eval_if::*;
pub use eval_ops::*;
pub use eval_parse_blocks::*;
pub use eval_print::*;
pub use eval_replace_symbol::*;
pub use eval_split_line::*;
pub use eval_strip_comments::*;
pub use eval_ternary::*;
pub use eval_unraw::*;

//====================================================================================================================//
// Eval Context
//====================================================================================================================//

/// Context for an [`NodeEval`] execution.
pub struct EvalContext {
	scope: Scope,
	declares: Vec<(Symbol, CodeOffset, Node)>,
}

impl EvalContext {
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
		// TODO: review what the value of the declare actually means in the code
		self.declares.push((symbol, offset, value));
	}

	pub(crate) fn get_declares(&mut self) -> Vec<(Symbol, CodeOffset, Node)> {
		std::mem::take(&mut self.declares)
	}
}

//====================================================================================================================//
// Node eval
//====================================================================================================================//

/// An operation applicable to a [`Node`] and [`Scope`].
pub trait IsNodeEval {
	fn applies(&self, node: &Node) -> bool;

	fn execute(&self, ctx: &mut EvalContext, node: &mut Node) -> Result<()>;
}

/// Evaluation order precedence for [`NodeEval`] evaluation.
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
pub enum NodeEval {
	Brackets(BracketPairs),
	Block(Symbol),
	SplitLines,
	StripComments,
	Let(Symbol, Symbol, Decl),
	If(Symbol, Symbol),
	For(Symbol, Symbol, Symbol),
	Ternary(EvalTernary),
	Print(Symbol),
	Comma(Symbol),
	Replace(Symbol, fn(ScopeHandle, Span) -> Node),
	Bind,
	ParseExpression(OperatorSet),
	Unraw,
}

impl NodeEval {
	pub fn get_impl(&self) -> Arc<dyn IsNodeEval> {
		match self {
			NodeEval::Brackets(pairs) => Arc::new(pairs.clone()),
			NodeEval::Block(symbol) => Arc::new(EvalParseBlocks(symbol.clone())),
			NodeEval::SplitLines => Arc::new(EvalSplitLine),
			NodeEval::StripComments => Arc::new(EvalStripComments),
			NodeEval::Let(decl, eq, mode) => Arc::new(EvalDecl(decl.clone(), eq.clone(), *mode)),
			NodeEval::If(s_if, s_else) => Arc::new(EvalIf::new(s_if.clone(), s_else.clone())),
			NodeEval::For(a, b, c) => Arc::new(EvalFor(a.clone(), b.clone(), c.clone())),
			NodeEval::Bind => Arc::new(EvalBind),
			NodeEval::Print(symbol) => Arc::new(EvalPrint(symbol.clone())),
			NodeEval::Replace(symbol, node) => Arc::new(ReplaceSymbol(symbol.clone(), node.clone())),
			NodeEval::ParseExpression(ops) => Arc::new(ops.clone()),
			NodeEval::Comma(symbol) => Arc::new(SplitComma(symbol.clone())),
			NodeEval::Ternary(op) => Arc::new(op.clone()),
			NodeEval::Unraw => Arc::new(EvalUnraw),
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
	Member,
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
// Evaluators
//====================================================================================================================//

pub fn configure_default_node_evaluators(scope: &mut ScopeWriter) {
	// expression parsing
	let ops = default_operators();

	let mut matcher = scope.matcher();
	ops.register_symbols(&mut matcher);
	scope.set_matcher(matcher);

	scope.add_node_eval(NodeEval::ParseExpression(ops), NodePrecedence::Expression);
	scope.add_node_eval(NodeEval::Unraw, NodePrecedence::Least);

	//general parsing
	scope.add_node_eval(NodeEval::Block(Context::symbol(":")), NodePrecedence::Blocks);
	scope.add_node_eval(NodeEval::SplitLines, NodePrecedence::SplitLines);
	scope.add_node_eval(NodeEval::StripComments, NodePrecedence::Comments);
	scope.add_node_eval(
		NodeEval::Let(Context::symbol("let"), Context::symbol("="), Decl::Let),
		NodePrecedence::Let,
	);
	scope.add_node_eval(
		NodeEval::Let(Context::symbol("const"), Context::symbol("="), Decl::Const),
		NodePrecedence::Const,
	);
	scope.add_node_eval(
		NodeEval::If(Context::symbol("if"), Context::symbol("else")),
		NodePrecedence::If,
	);
	scope.add_node_eval(
		NodeEval::For(Context::symbol("for"), Context::symbol("in"), Context::symbol("..")),
		NodePrecedence::For,
	);
	scope.add_node_eval(NodeEval::Bind, NodePrecedence::Bind);
	scope.add_node_eval(NodeEval::Print(Context::symbol("print")), NodePrecedence::Print);
	scope.add_node_eval(NodeEval::Comma(Context::symbol(",")), NodePrecedence::Comma);

	let ternary = EvalTernary(
		Context::symbol("?"),
		Context::symbol(":"),
		Arc::new(|a, b, c, scope, span| Expr::Conditional(a, b, c).at(scope, span)),
	);
	scope.add_node_eval(NodeEval::Ternary(ternary), NodePrecedence::Ternary);

	// brackets
	let mut brackets = BracketPairs::new();
	brackets.add(
		Context::symbol("("),
		Context::symbol(")"),
		Arc::new(|_, n, _| Expr::Group(n)),
	);

	scope.add_node_eval(NodeEval::Brackets(brackets), NodePrecedence::Brackets);

	// TODO: handle literal values properly as to not need different precedences

	// boolean
	scope.add_node_eval(
		NodeEval::Replace(Context::symbol("true"), |scope, span| {
			Expr::Boolean(true).at(scope, span)
		}),
		NodePrecedence::Boolean(true),
	);
	scope.add_node_eval(
		NodeEval::Replace(Context::symbol("false"), |scope, span| {
			Expr::Boolean(false).at(scope, span)
		}),
		NodePrecedence::Boolean(false),
	);

	// null
	scope.add_node_eval(
		NodeEval::Replace(Context::symbol("null"), |scope, span| Expr::Null.at(scope, span)),
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

	ops.add(Operator::new_binary(
		".".into(),
		BinaryOp::Member,
		OpPrecedence::Member,
		Grouping::Left,
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
// Evaluator binding logic
//====================================================================================================================//

/*
	NOTE ON PERFORMANCE
	===================

	The algorithm below is absolutely abhorrent and is the core algorithm that
	runs for parsing the input source. Having this be the fastest it can is
	crucial for the compilation performance.

	Ideas to improve performance:

	- Each evaluator should keep an index of applicable nodes. Finding the next
	  applicable evaluator and its set of nodes would be O(1).

		- We need to keep a set of active (non-empty) evaluators sorted by index.
		- The index does not need to be perfect, as that would be very hard to
		  keep. Even with false positives, it should allow quickly culling the
		  set of nodes for an evaluator and quickly identifying no-ops.
		- Nodes that have been applied an evaluator, should be saved so that they
		  are out of consideration for that evaluator.
			- This could be as simple as saving the last processed node ID
			  for each evaluator (assuming Node IDs are strictly incremental).

	- As nodes are created, we need a fast lookup function to identify which
	  evaluators could apply to a node. In some cases, it might make more sense
	  to keep an index of specific node types instead.

	- The index above would not be perfect, so evaluators still need to check
	  their apply condition to nodes. That should be done only for the highest
	  precedence evaluator globally.

	- We might need a quick way of removing nodes from all indexes (e.g. once
	  they have been absorbed into another node). This could be a culling step
	  in the apply condition.

	- Consider having the tree of nodes as a flattened list of segments. A lot
	  of the nodes would consist of those (e.g. tokens, lines, groups, block
	  segments).

	- Quickly looking up parent info for some of the nodes can be crucial for
	  some of the parsing evaluators.

	- A lot of the evaluator can apply locally by simply replacing a node value.
	  This should be a very fast operation.

	Minor:

	- Most scopes won't change the set of evaluators applicable, so there should
	  be a fast path for that.

*/

pub struct NodeOperation {
	node: Node,
	location: NodeLocation,
	evaluator: NodeEval,
	evaluator_impl: Arc<dyn IsNodeEval>,
}

impl NodeOperation {
	pub fn node(&self) -> &Node {
		&self.node
	}

	pub fn evaluator(&self) -> &NodeEval {
		&self.evaluator
	}

	pub fn evaluator_impl(&self) -> &dyn IsNodeEval {
		&*self.evaluator_impl
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
	// TODO: allow operations that are defined by the node itself
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

		// collect all evaluators for child nodes
		let (mut operations, mut max_precedence) = {
			let mut cur_precedence = max_precedence;
			let mut cur_ops = Vec::new();
			let mut location = location.push(self.clone());
			for (index, it) in self.expr().children().into_iter().enumerate() {
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

		// collect evaluators that apply to this node
		let evaluators = self.scope().get_node_evaluators().into_iter().filter_map(|(op, prec)| {
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

		let mut evaluators = evaluators.take_while(|(.., prec)| {
			prec != &NodePrecedence::Never
				&& if let Some(max) = max_precedence {
					prec <= &max
				} else {
					true
				}
		});

		// do we have an evaluator for this node?
		if let Some((op, op_impl, prec)) = evaluators.next() {
			// validate that we have only one applicable evaluator
			let evaluators = evaluators.take_while(|(.., op_prec)| op_prec == &prec && prec != NodePrecedence::Never);
			let evaluators = evaluators.collect::<Vec<_>>();
			if evaluators.len() > 0 {
				let mut error = format!("node can accept multiple node evaluators at the same precedence\n-> {op:?}");
				for (op, ..) in evaluators {
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
					evaluator: op,
					evaluator_impl: op_impl,
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
