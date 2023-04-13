use std::any::TypeId;
use std::fmt::Display;
use std::sync::Arc;

use super::*;

pub trait IsType: 'static + Display {
	fn type_id(&self) -> TypeId;

	fn fmt_val(&self, value: &InnerValue, f: &mut std::fmt::Formatter) -> std::fmt::Result;

	fn drop_val(&self, value: &mut InnerValue);

	fn clone_val(&self, value: &InnerValue) -> InnerValue;
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Type {
	Unit,
	Never,
	Bool,
	Int(TypeInt),
	Float(TypeFloat),
	Other(OtherType),
}

impl Type {
	fn new<T: IsType>(typ: T) -> Self {
		let typ = Box::new(typ);
		let typ = Box::leak(typ) as *const dyn IsType;
		Type::Other(OtherType(typ))
	}
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum TypeInt {
	Any,
	I8,
	U8,
	I16,
	I32,
	I64,
	U16,
	U32,
	U64,
	ISize,
	USize,
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum TypeFloat {
	Any,
	F32,
	F64,
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct OtherType(*const dyn IsType);

impl OtherType {
	pub fn get(&self) -> &dyn IsType {
		unsafe { self.0.as_ref().unwrap() }
	}
}
