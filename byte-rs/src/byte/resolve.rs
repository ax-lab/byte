use std::cmp::Ordering;

use super::*;

impl Compiler {
	pub fn resolve_next(&self, node_list: &NodeList, errors: &mut Errors) -> Option<NodeList> {
		// filter nodes that can be evaluated
		let mut nodes = node_list
			.iter()
			.enumerate()
			.filter_map(|(pos, node)| {
				let node = node.value();
				if let Some(prec) = node.precedence() {
					Some((prec, pos, node))
				} else {
					None
				}
			})
			.collect::<Vec<_>>();

		// sort nodes by precedence
		nodes.sort_by(|((prec1, seq1), pos1, ..), ((prec2, seq2), pos2, ..)| {
			let order = prec1.cmp(prec2);
			if order == Ordering::Equal {
				if seq1 != seq2 {
					// nodes with different sequencing groups are evaluated
					// within their own groups
					return seq1.cmp(&seq2);
				}
				let (a, b) = match seq1 {
					Sequence::Ordered => (*pos1, *pos2),
					Sequence::Reverse => (*pos2, *pos1),
					Sequence::AtOnce => (0, 0),
				};
				a.cmp(&b)
			} else {
				order
			}
		});

		let (first_prec, first_seq) = if let Some(((prec, seq), ..)) = nodes.first() {
			(prec, seq)
		} else {
			// nothing to evaluate
			return None;
		};

		// filter nodes to evaluate
		let nodes = nodes
			.iter()
			.enumerate()
			.take_while(|(n, item)| {
				let ((prec, ..), ..) = item;
				*n == 0 || prec == first_prec && first_seq == &Sequence::AtOnce
			})
			.map(|(index, (.., node))| (index, node));

		let mut changes = Vec::new();
		for (index, node) in nodes {
			let mut ctx = ResolveContext {
				compiler: self,
				errors: Default::default(),
				changes: Default::default(),
				nodes: node_list,
				index,
			};

			node.evaluate(&mut ctx);
			if ctx.has_changes() {
				changes.push(ctx);
			}
		}

		ResolveContext::apply_changes(node_list, changes, errors)
	}
}

//====================================================================================================================//
// Context
//====================================================================================================================//

pub struct ResolveContext<'a> {
	compiler: &'a Compiler,
	errors: Errors,
	changes: Vec<Change>,
	nodes: &'a NodeList,
	index: usize,
}

impl<'a> ResolveContext<'a> {
	pub fn compiler(&self) -> &Compiler {
		self.compiler
	}

	pub fn errors_mut(&mut self) -> &mut Errors {
		&mut self.errors
	}

	pub fn replace_self<I: IntoIterator<Item = Node>>(&mut self, nodes: I) {
		self.replace_range(self.index..self.index + 1, nodes)
	}

	pub fn replace_range<T: RangeBounds<usize>, I: IntoIterator<Item = Node>>(&mut self, range: T, nodes: I) {
		let range = compute_range(range, self.nodes.len());
		assert!(range.end <= self.nodes.len() && range.start <= range.end);
		self.push_change(Change::Replace {
			index: range.start,
			count: range.end - range.start,
			nodes: nodes.into_iter().collect(),
		});
	}

	fn has_changes(&self) -> bool {
		self.errors.len() > 0 || self.changes.len() > 0
	}

	fn push_change(&mut self, new_change: Change) {
		for it in self.changes.iter() {
			if let Some(error) = it.check_conflict(&new_change) {
				panic!("invalid change: {error}")
			}
		}
		self.changes.push(new_change);
	}

	fn apply_changes(node_list: &NodeList, changes: Vec<ResolveContext>, errors: &mut Errors) -> Option<NodeList> {
		for it in changes.iter() {
			errors.append(&it.errors);
		}

		let mut changes = changes
			.into_iter()
			.map(|x| (x.index, x.changes))
			.flat_map(|(n, x)| x.into_iter().map(move |x| (n, x)))
			.collect::<Vec<_>>();

		// TODO: this could be optimized by considering overlaps while replacing
		for i in 0..changes.len() - 1 {
			for j in i + 1..changes.len() {
				let (na, xa) = &changes[i];
				let (nb, xb) = &changes[j];
				if let Some(error) = xa.check_conflict(xb) {
					let na = &node_list[*na];
					let nb = &node_list[*nb];
					let sa = fmt_indented_debug(&na, "  - ", "    ");
					let sb = fmt_indented_debug(&nb, "  - ", "    ");
					let error = format!("node eval conflict: {error}\n{sa}\n{sb}");
					errors.add_at(error, na.span().or_else(|| nb.span()).cloned());
				}
			}
		}

		changes.sort_by_key(|(_, change)| {
			let Change::Replace { index, .. } = change;
			*index
		});

		let mut result = Vec::new();
		let mut cursor = 0;
		for Change::Replace { index, count, nodes } in changes.into_iter().map(|x| x.1) {
			if index > cursor {
				result.extend_from_slice(node_list.slice(cursor..index));
				cursor = index;
			}
			assert!(index >= cursor); // overlapping changes

			result.extend(nodes);
			cursor = std::cmp::max(cursor, index + count);
		}

		result.extend_from_slice(node_list.slice(cursor..));

		Some(NodeList::new(result))
	}
}

//====================================================================================================================//
// Helpers
//====================================================================================================================//

enum Change {
	Replace {
		index: usize,
		count: usize,
		nodes: Vec<Node>,
	},
}

#[allow(irrefutable_let_patterns)]
impl Change {
	pub fn check_conflict(&self, other: &Change) -> Option<String> {
		if let Change::Replace { index, count, .. } = self {
			let (a1, a2) = (*index, index + count);
			if let Change::Replace { index, count, .. } = other {
				let (b1, b2) = (*index, index + count);
				if a1 < b2 && b1 < a2 {
					return Some(format!("modified ranges #{a1}…{a2} and #{b1}…{b2} overlap"));
				} else if a1 == b1 && a2 == b2 {
					return Some(format!("position #{a1} modified more than once"));
				}
			}
		}
		None
	}
}
