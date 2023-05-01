use std::fmt::{Debug, Display, Formatter};

use super::*;

pub trait IsError: HasRepr {
	fn span(&self) -> Option<Span>;
}

pub struct Errors {}

impl Debug for Errors {
	fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
		todo!()
	}
}

impl Display for Errors {
	fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
		todo!()
	}
}
