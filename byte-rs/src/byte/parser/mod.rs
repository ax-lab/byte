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
