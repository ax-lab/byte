use super::*;

pub mod decl;
pub mod indent;
pub mod parse_ops;
pub mod print;
pub mod ternary;

pub use decl::*;
pub use indent::*;
pub use parse_ops::*;
pub use print::*;
pub use ternary::*;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum NodeOperator {
	Brackets(BracketPairs, NodePrecedence),
	SplitLines(NodePrecedence),
	Let(NodePrecedence),
	Ternary(TernaryOp, NodePrecedence),
	Print(NodePrecedence),
	Comma(Symbol, NodePrecedence),
	Replace(Symbol, fn(Span) -> Node, NodePrecedence),
	Bind(NodePrecedence),
	Binary(ParseBinaryOp, NodePrecedence),
	UnaryPrefix(ParseUnaryPrefixOp, NodePrecedence),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Grouping {
	Left,
	Right,
}

impl NodeOperator {
	pub fn precedence(&self) -> NodePrecedence {
		self.get_impl().1
	}

	pub fn can_apply(&self, nodes: &NodeList) -> bool {
		self.get_impl().0.can_apply(nodes)
	}

	pub fn apply(&self, nodes: &mut NodeList, context: &mut EvalContext) -> Result<()> {
		self.get_impl().0.apply(nodes, context)
	}

	fn get_impl(&self) -> (Arc<dyn IsNodeOperator>, NodePrecedence) {
		match self {
			NodeOperator::SplitLines(prec) => (Arc::new(OpSplitLine), *prec),
			NodeOperator::Let(prec) => (Arc::new(LetOperator), *prec),
			NodeOperator::Bind(prec) => (Arc::new(OpBind), *prec),
			NodeOperator::Print(prec) => (Arc::new(PrintOperator), *prec),
			NodeOperator::Replace(symbol, node, prec) => (Arc::new(ReplaceSymbol(symbol.clone(), node.clone())), *prec),
			NodeOperator::Binary(op, prec) => (Arc::new(op.clone()), *prec),
			NodeOperator::UnaryPrefix(op, prec) => (Arc::new(op.clone()), *prec),
			NodeOperator::Comma(symbol, prec) => (Arc::new(CommaOperator(symbol.clone())), *prec),
			NodeOperator::Brackets(pairs, prec) => (Arc::new(pairs.clone()), *prec),
			NodeOperator::Ternary(op, prec) => (Arc::new(op.clone()), *prec),
		}
	}
}
