//! Miscellaneous utility code for the compiler.

pub mod format;
pub use format::*;

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

pub struct Timer {
	timer: std::time::Instant,
	label: String,
}

pub fn measure<T: Into<String>>(label: T) -> Timer {
	let timer = std::time::Instant::now();
	let label = label.into();
	Timer { label, timer }
}

impl Timer {
	pub fn elapsed(&self, point: &str) {
		let label = &self.label;
		println!("{label}: {:?} ({point})", self.timer.elapsed())
	}
}
