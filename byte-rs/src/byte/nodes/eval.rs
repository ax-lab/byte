use super::*;

/// An operation applicable to a [`NodeList`] and [`Scope`].
pub trait Evaluator {
	fn apply(&self, scope: &Scope, nodes: &mut Vec<Node>, context: &mut EvalContext) -> Result<bool>;

	fn can_apply(&self, nodes: &NodeList) -> bool {
		nodes.contains(|x| self.predicate(x))
	}

	fn predicate(&self, node: &Node) -> bool {
		let _ = node;
		false
	}
}

/// Evaluation order precedence for [`Evaluator`].
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum EvalPrecedence {
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

//====================================================================================================================//
// Context
//====================================================================================================================//

/// Context for an [`Evaluator`] application.
pub struct EvalContext {
	span: Span,
	new_segments: Vec<NodeList>,
	declares: Vec<(Symbol, Option<usize>, BindingValue)>,
}

impl EvalContext {
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
