use std::any::TypeId;
use std::sync::Arc;

use crate::core::num::*;
use crate::core::*;

use super::*;

pub trait IsType: IsValue {
	fn get_as_type(&self) -> &dyn IsType;
}

#[derive(Clone, Eq, PartialEq)]
pub enum Type {
	Unit,
	Never,
	Bool,
	String,
	Int(kind::Int),
	Float(kind::Float),
	Other(OtherType),
}

impl Type {
	pub fn new<T: IsType>(typ: T) -> Self {
		let typ = Value::from(typ);
		Type::Other(OtherType(typ))
	}
}

#[derive(Clone, Eq, PartialEq)]
pub struct OtherType(Value);

impl OtherType {
	pub fn new<T: IsType>(value: T) -> Self {
		let value = Value::from(value);
		assert!(get_trait!(&value, IsType).is_some());
		OtherType(value)
	}
	pub fn get(&self) -> &dyn IsType {
		get_trait!(&self.0, IsType).unwrap()
	}
}

impl std::fmt::Debug for OtherType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "Type({:?})", self.0)
	}
}
