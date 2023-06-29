use super::*;

pub mod bracket;
pub mod indent;
pub mod line;
pub mod op_binary;
pub mod op_ternary;
pub mod op_unary;

pub use bracket::*;
pub use indent::*;
pub use line::*;
pub use op_binary::*;
pub use op_ternary::*;
pub use op_unary::*;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum Operator {
	Tokenize,
	SplitLines,
}

impl Operator {
	pub fn precedence(&self) -> Precedence {
		match self {
			Operator::Tokenize => Precedence::Lexer,
			Operator::SplitLines => Precedence::LineSplit,
		}
	}

	pub fn can_apply(&self, nodes: &NodeList) -> bool {
		match self {
			Operator::Tokenize => nodes.contains(|x| matches!(x, Node::RawText(..))),
			Operator::SplitLines => nodes.contains(|x| x == &Node::Break),
		}
	}

	pub fn apply(&self, context: &mut OperatorContext, errors: &mut Errors) {
		let _ = (context, errors);
		todo!()
	}
}

/// Global evaluation precedence for language nodes.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum Precedence {
	First,
	Lexer,
	LineSplit,
	Last,
}

pub struct OperatorContext<'a> {
	nodes: &'a mut NodeList,
	changes: NodeChanges,
}

#[derive(Default)]
pub struct NodeChanges {
	moves: Vec<(Range<usize>, Vec<NodeData>)>,
}

impl<'a> OperatorContext<'a> {
	pub fn new(nodes: &'a mut NodeList) -> Self {
		Self {
			nodes,
			changes: Default::default(),
		}
	}

	pub fn has_changes(&self) -> bool {
		self.changes.moves.len() > 0
	}

	pub fn nodes(&self) -> &NodeList {
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
