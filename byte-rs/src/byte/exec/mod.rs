pub use std::io::Write;

use super::code::*;
use super::core::*;
use super::runtime::*;

pub mod error;
pub mod eval;

pub use error::*;

pub type Result<T> = std::result::Result<T, Error>;

impl Code {
	pub fn execute(&self, _rt: &mut Runtime) -> Result<Value> {
		todo!()
	}
}
