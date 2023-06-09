pub mod code;
pub mod context;
pub mod eval;
pub mod lexer;
pub mod names;
pub mod nodes;
pub mod precedence;
pub mod source;
pub mod util;

pub use context::*;
pub use lexer::*;
pub use names::*;
pub use nodes::*;
pub use precedence::*;
pub use source::*;
pub use util::*;

pub type Result<T> = std::result::Result<T, Errors>;

use std::{
	fmt::{Debug, Display, Formatter, Write},
	ops::Deref,
	sync::Arc,
};
