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
	Brackets(BracketPairs, EvalPrecedence),
	SplitLines(EvalPrecedence),
	Let(EvalPrecedence),
	Ternary(TernaryOp, EvalPrecedence),
	Print(EvalPrecedence),
	Comma(EvalPrecedence),
	Replace(Symbol, fn(Span) -> Node, EvalPrecedence),
	Bind(EvalPrecedence),
	Binary(ParseBinaryOp, EvalPrecedence),
	UnaryPrefix(ParseUnaryPrefixOp, EvalPrecedence),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Grouping {
	Left,
	Right,
}

impl Operator {
	pub fn precedence(&self) -> EvalPrecedence {
		self.get_impl().1
	}

	pub fn can_apply(&self, nodes: &NodeList) -> bool {
		self.get_impl().0.can_apply(nodes)
	}

	pub fn apply(&self, nodes: &mut NodeList, context: &mut EvalContext) -> Result<()> {
		self.get_impl().0.apply(nodes, context)
	}

	fn get_impl(&self) -> (Arc<dyn Evaluator>, EvalPrecedence) {
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
