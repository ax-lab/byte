use super::*;

pub mod bind;
pub mod bracket;
pub mod comma;
pub mod decl;
pub mod indent;
pub mod line;
pub mod parse_ops;
pub mod print;
pub mod replace_symbol;
pub mod ternary;

pub use bind::*;
pub use bracket::*;
pub use comma::*;
pub use decl::*;
pub use indent::*;
pub use line::*;
pub use parse_ops::*;
pub use print::*;
pub use replace_symbol::*;
pub use ternary::*;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum Operator {
	Brackets(BracketPairs, Precedence),
	SplitLines(Precedence),
	Let(Precedence),
	Ternary(TernaryOp, Precedence),
	Print(Precedence),
	Comma(Precedence),
	Replace(Symbol, fn(Span) -> Node, Precedence),
	Bind(Precedence),
	Binary(ParseBinaryOp, Precedence),
	UnaryPrefix(ParseUnaryPrefixOp, Precedence),
}

/// Global evaluation precedence for language nodes.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Precedence {
	Highest,
	Brackets,
	SplitLines,
	Let,
	Print,
	Ternary,
	Comma,
	OpAssign,
	OpUnaryPrefix, // FIXME: this needs to be parsed properly
	OpBooleanOr,
	OpBooleanAnd,
	OpAdditive,
	OpMultiplicative,
	Boolean(bool),
	Null,
	Bind,
	Least,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Grouping {
	Left,
	Right,
}

impl Operator {
	pub fn precedence(&self) -> Precedence {
		self.get_impl().1
	}

	pub fn can_apply(&self, nodes: &NodeList) -> bool {
		self.get_impl().0.can_apply(nodes)
	}

	pub fn apply(&self, scope: &Scope, nodes: &mut Vec<Node>, context: &mut OperatorContext) -> Result<bool> {
		self.get_impl().0.apply(scope, nodes, context)
	}

	fn get_impl(&self) -> (Arc<dyn IsOperator>, Precedence) {
		match self {
			Operator::SplitLines(prec) => (Arc::new(SplitLineOperator), *prec),
			Operator::Let(prec) => (Arc::new(LetOperator), *prec),
			Operator::Bind(prec) => (Arc::new(BindOperator), *prec),
			Operator::Print(prec) => (Arc::new(PrintOperator), *prec),
			Operator::Replace(symbol, node, prec) => (Arc::new(ReplaceSymbol(symbol.clone(), node.clone())), *prec),
			Operator::Binary(op, prec) => (Arc::new(op.clone()), *prec),
			Operator::UnaryPrefix(op, prec) => (Arc::new(op.clone()), *prec),
			Operator::Comma(prec) => (Arc::new(CommaOperator), *prec),
			Operator::Brackets(pairs, prec) => (Arc::new(pairs.clone()), *prec),
			Operator::Ternary(op, prec) => (Arc::new(op.clone()), *prec),
		}
	}
}

/*
	TODO: implement operators protocol

	- Operators should support application to any NodeList and any number of
	  times.

	- When an operator makes no change to the NodeList or scope, then it's
	  considered as not applicable.

	- For performance reasons, operators should prune themselves from
	  application as soon and as quick as possible.

	- Performance: keep a set of applicable operators for a NodeList and
	  remove operators as they are applied. When the list changes, check if
	  any new node triggers an operator.

	- Some operators may require complex parsing, at the end of which may
	  result in the operator no applying. Operator application should be a two
	  step process of parsing and committing.

	- Keep a dirty flag for node ranges to detect if multiple operators try to
	  change the same set of nodes. This would allow multiple operators with
	  the same precedence, as long as they act on separate nodes.

*/

pub trait IsOperator {
	fn apply(&self, scope: &Scope, nodes: &mut Vec<Node>, context: &mut OperatorContext) -> Result<bool>;

	fn can_apply(&self, nodes: &NodeList) -> bool {
		nodes.contains(|x| self.predicate(x))
	}

	fn predicate(&self, node: &Node) -> bool {
		let _ = node;
		false
	}
}

pub struct OperatorContext {
	span: Span,
	new_segments: Vec<NodeList>,
	declares: Vec<(Symbol, Option<usize>, BindingValue)>,
}

impl OperatorContext {
	pub fn new(span: Span) -> Self {
		Self {
			span,
			new_segments: Default::default(),
			declares: Default::default(),
		}
	}

	pub fn span(&self) -> Span {
		self.span.clone()
	}

	pub fn resolve_nodes(&mut self, list: &NodeList) {
		self.new_segments.push(list.clone())
	}

	pub fn declare_static(&mut self, symbol: Symbol, value: BindingValue) {
		self.declares.push((symbol, None, value));
	}

	pub fn declare_at(&mut self, symbol: Symbol, offset: usize, value: BindingValue) {
		self.declares.push((symbol, Some(offset), value));
	}

	pub(crate) fn get_new_segments(&mut self, output: &mut Vec<NodeList>) {
		output.append(&mut self.new_segments)
	}

	pub(crate) fn get_declares(&mut self) -> Vec<(Symbol, Option<usize>, BindingValue)> {
		std::mem::take(&mut self.declares)
	}
}
