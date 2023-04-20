pub mod any;
#[allow(unused)]
pub mod cell;
pub mod error;
pub mod input;
#[allow(unused)]
pub mod num;
pub mod str;
pub mod util;

pub use any::{has_traits, to_trait, HasTraits, IsValue};

pub mod kind {
	use super::*;

	pub use num::kind::*;
}
