pub mod declare;
pub mod node;
pub mod segments;

pub use node::*;
pub use segments::*;

use std::io::Write;

use crate::core::*;
use crate::lexer::*;

pub trait IsNode: IsValue + WithEquality {}
