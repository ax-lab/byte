//! Miscellaneous utility code for the compiler.

pub mod format;
pub mod traits;

pub use format::*;
pub use traits::*;

use super::*;

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
