//! Miscellaneous utility code for the compiler.

pub mod arena;
pub mod common;
pub mod errors;
pub mod format;
pub mod traits;
pub mod value;

pub use arena::*;
pub use common::*;
pub use errors::*;
pub use format::*;
pub use traits::*;
pub use value::*;

use super::*;

//====================================================================================================================//
// Ids
//====================================================================================================================//

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Id(usize);

impl Id {
	pub fn value(&self) -> usize {
		self.0
	}
}

/// Returns a globally unique, non-zero, increasing numeric ID.
pub fn new_id() -> Id {
	use std::sync::atomic::*;
	static COUNTER: AtomicUsize = AtomicUsize::new(1);
	let id = COUNTER.fetch_add(1, Ordering::SeqCst);
	Id(id)
}

impl Display for Id {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "[#{}]", self.0)
	}
}

//====================================================================================================================//
// Ranges
//====================================================================================================================//

pub fn compute_range<R: RangeBounds<usize>>(range: R, len: usize) -> Range<usize> {
	let sta = match range.start_bound() {
		std::ops::Bound::Included(n) => *n,
		std::ops::Bound::Excluded(n) => *n + 1,
		std::ops::Bound::Unbounded => 0,
	};
	let end = match range.end_bound() {
		std::ops::Bound::Included(n) => *n + 1,
		std::ops::Bound::Excluded(n) => *n,
		std::ops::Bound::Unbounded => len,
	};
	assert!(end <= len);
	assert!(sta <= end);
	sta..end
}
