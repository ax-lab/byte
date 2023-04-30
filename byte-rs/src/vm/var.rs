use crate::core::*;

use super::*;

#[derive(Clone)]
pub struct Var {
	val: Value,
	typ: Type,
}

impl Var {
	pub fn val(&self) -> Value {
		self.val.clone()
	}

	pub fn typ(&self) -> Type {
		self.typ.clone()
	}
}
