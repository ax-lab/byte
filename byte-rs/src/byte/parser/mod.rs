//! Provides a set of parsing operators that can be applied to [`NodeList`].

use super::*;

#[derive(Clone, Copy, Debug)]
pub enum NodeRange {
	At(usize),
	List { at: usize, len: usize },
}

// TODO: move node operators into their own files
// TODO: rethink NodeList interface for operators
// TODO: implement generic operators for other ops

pub trait SplitByNode {
	fn is_split(&self, node: &Node) -> bool;

	fn new_node(&self, scope: &Scope, segment: Vec<Node>) -> Result<Node>;
}

impl<T: SplitByNode> Evaluator for T {
	fn predicate(&self, node: &Node) -> bool {
		self.is_split(node)
	}

	fn apply(&self, scope: &Scope, nodes: &mut Vec<Node>, context: &mut EvalContext) -> Result<bool> {
		let mut new_nodes = Vec::new();
		let mut line = Vec::new();

		for it in nodes.iter() {
			if self.is_split(it) {
				let node = self.new_node(scope, std::mem::take(&mut line))?;
				node.bit().get_dependencies(|list| context.resolve_nodes(list));
				new_nodes.push(node);
			} else {
				line.push(it.clone());
			}
		}

		if line.len() > 0 {
			let node = self.new_node(scope, std::mem::take(&mut line))?;
			node.bit().get_dependencies(|list| context.resolve_nodes(list));
			new_nodes.push(node);
		}

		let changed = if new_nodes.len() > 0 {
			*nodes = new_nodes;
			true
		} else {
			false
		};
		Ok(changed)
	}
}

pub trait BracketInfo {
	fn is_open(&self) -> bool;

	fn is_close(&self) -> bool {
		!self.is_open()
	}
}

pub trait NodeGrouper {
	type Bracket: BracketInfo;

	fn find_bracket(&self, nodes: &NodeList) -> Option<(usize, Self::Bracket)>;

	fn validate_pair(&self, open: &Node, close: &Node) -> Result<()>;

	fn err_missing_close(&self, open: &Node) -> Errors {
		Errors::from(format!("missing close bracket for `{open}`"), open.span())
	}

	fn err_missing_open(&self, close: &Node) -> Errors {
		Errors::from(format!("unmatched closing bracket `{close}`"), close.span())
	}
}
