//! Miscellaneous utility code for the compiler.

pub mod arena;
pub mod common;
pub mod errors;
pub mod format;
pub mod handle;
pub mod traits;
pub mod value;

pub use arena::*;
pub use common::*;
pub use errors::*;
pub use format::*;
pub use handle::*;
pub use traits::*;
pub use value::*;

use super::*;

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
