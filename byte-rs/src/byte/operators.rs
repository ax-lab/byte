use super::*;

pub mod bracket;
pub mod indent;
pub mod line;
pub mod module;
pub mod op_binary;
pub mod op_ternary;
pub mod op_unary;

pub use bracket::*;
pub use indent::*;
pub use line::*;
pub use module::*;
pub use op_binary::*;
pub use op_ternary::*;
pub use op_unary::*;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum Operator {
	Module,
	Tokenize,
	SplitLines,
}

/// Global evaluation precedence for language nodes.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Precedence {
	First,
	Modules,
	Lexer,
	LineSplit,
	Last,
}

impl Operator {
	pub fn precedence(&self) -> Precedence {
		self.get_impl().precedence()
	}

	pub fn can_apply(&self, nodes: &NodeList) -> bool {
		self.get_impl().can_apply(nodes)
	}

	pub fn apply(&self, context: &mut OperatorContext, errors: &mut Errors) {
		self.get_impl().apply(context, errors)
	}

	fn get_impl(&self) -> &dyn IsOperator {
		match self {
			Operator::Module => &ModuleOperator,
			Operator::Tokenize => todo!(),
			Operator::SplitLines => todo!(),
		}
	}
}

pub trait IsOperator {
	fn precedence(&self) -> Precedence;

	fn apply(&self, context: &mut OperatorContext, errors: &mut Errors);

	fn can_apply(&self, nodes: &NodeList) -> bool {
		nodes.contains(|x| self.predicate(x))
	}

	fn predicate(&self, node: &Node) -> bool {
		let _ = node;
		false
	}
}

pub struct OperatorContext<'a> {
	nodes: &'a mut NodeList,
	scope: HandleRef<Scope>,
	changes: NodeChanges,
}

#[derive(Default)]
pub struct NodeChanges {
	moves: Vec<(Range<usize>, Vec<NodeData>)>,
}

impl<'a> OperatorContext<'a> {
	pub fn new(nodes: &'a mut NodeList) -> Self {
		let scope = nodes.scope();
		Self {
			nodes,
			scope,
			changes: Default::default(),
		}
	}

	pub fn has_changes(&self) -> bool {
		self.changes.moves.len() > 0
	}

	pub fn scope(&self) -> &Scope {
		&self.scope
	}

	pub fn nodes(&mut self) -> &mut NodeList {
		self.nodes
	}

	pub fn replace_nodes<T: RangeBounds<usize>>(&mut self, range: T, nodes: Vec<NodeData>) {
		let range = compute_range(range, self.nodes.len());

		// find the index of the first existing range that is either past the
		// new range starting point, or overlaps it
		let index = self
			.changes
			.moves
			.partition_point(|x| x.0.start < range.start && x.0.end <= range.start);

		// validate that the new range does not overlap the existing range
		if index < self.changes.moves.len() {
			let next = &self.changes.moves[index].0;

			// handle the new range overlapping the existing one or both being
			// zero-width insertions at the same point
			let overlaps = range.end > next.start || &range == next;
			if overlaps {
				let (r1, r2) = (range.start, range.end);
				let (n1, n2) = (next.start, next.end);
				panic!("operator replacement #{r1}-{r2} overlaps existing #{n1}-{n2}");
			}
		}
		self.changes.moves.insert(index, (range, nodes));
	}
}
