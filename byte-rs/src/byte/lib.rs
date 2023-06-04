pub mod code;
pub mod eval;
pub mod names;
pub mod util;

pub use names::*;
pub use util::*;

pub type Result<T> = std::result::Result<T, Errors>;

use std::fmt::{Debug, Formatter, Write};
