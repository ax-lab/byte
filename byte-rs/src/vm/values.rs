use std::any::{Any, TypeId};

use crate::core::*;

use super::*;

pub trait IsValue: 'static + Send + Sync {}

pub struct Value(pub Type, pub InnerValue);

impl Clone for Value {
	fn clone(&self) -> Self {
		let val = match &self.0 {
			Type::Other(typ) => typ.get().clone_val(&self.1),
			_ => self.1.clone(),
		};
		Self(self.0.clone(), val)
	}
}

impl Drop for Value {
	fn drop(&mut self) {
		if let Type::Other(typ) = self.0 {
			typ.get().drop_val(&mut self.1);
		}
	}
}

impl Value {
	pub fn unit() -> Value {
		Value(Type::Unit, InnerValue::default())
	}

	pub fn bool(value: bool) -> Value {
		Value(Type::Bool, InnerValue { bool: value })
	}

	pub fn typ(&self) -> &Type {
		&self.0
	}

	pub fn val(&self) -> &InnerValue {
		&self.1
	}

	pub fn val_mut(&mut self) -> &mut InnerValue {
		&mut self.1
	}

	pub fn as_ref<T: IsValue>(&self) -> Option<&T> {
		match self.typ() {
			Type::Other(typ) => {
				if typ.get().val_type_id() == TypeId::of::<T>() {
					unsafe { self.val().as_ref::<T>() }
				} else {
					None
				}
			}
			_ => None,
		}
	}

	pub fn as_mut<T: IsValue>(&mut self) -> Option<&mut T> {
		match self.typ() {
			Type::Other(typ) => {
				if typ.type_id() == TypeId::of::<T>() {
					unsafe { self.val_mut().as_mut::<T>() }
				} else {
					None
				}
			}
			_ => None,
		}
	}
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union InnerValue {
	pub bool: bool,
	pub int: num::Int,
	pub float: num::Float,
	pub ptr: *mut (),
}

impl Default for InnerValue {
	fn default() -> Self {
		unsafe { std::mem::zeroed() }
	}
}

impl InnerValue {
	pub fn pack<T: IsValue>(value: T) -> InnerValue {
		let ptr = Box::new(value);
		let ptr = Box::into_raw(ptr) as *mut ();
		InnerValue { ptr }
	}

	pub unsafe fn unpack<T: IsValue>(self) -> T {
		let ptr = self.ptr as *mut T;
		let ptr = Box::from_raw(ptr);
		*ptr
	}

	pub unsafe fn as_ref<T: IsValue>(&self) -> Option<&T> {
		let ptr = self.ptr as *const T;
		ptr.as_ref()
	}

	pub unsafe fn as_mut<T: IsValue>(&mut self) -> Option<&mut T> {
		let ptr = self.ptr as *mut T;
		ptr.as_mut()
	}
}

impl num::Int {
	pub fn new(typ: num::kind::Int, val: num::Int) -> Value {
		Value(Type::Int(typ), InnerValue { int: val })
	}

	pub fn i8(value: i8) -> Value {
		Self::new(num::kind::Int::I8, num::Int { i8: value })
	}

	pub fn i16(value: i16) -> Value {
		Self::new(num::kind::Int::I16, num::Int { i16: value })
	}

	pub fn i32(value: i32) -> Value {
		Self::new(num::kind::Int::I32, num::Int { i32: value })
	}

	pub fn i64(value: i64) -> Value {
		Self::new(num::kind::Int::I64, num::Int { i64: value })
	}

	pub fn isize(value: isize) -> Value {
		Self::new(num::kind::Int::ISize, num::Int { isize: value })
	}

	pub fn u8(value: u8) -> Value {
		Self::new(num::kind::Int::U8, num::Int { u8: value })
	}

	pub fn u16(value: u16) -> Value {
		Self::new(num::kind::Int::U16, num::Int { u16: value })
	}

	pub fn u32(value: u32) -> Value {
		Self::new(num::kind::Int::U32, num::Int { u32: value })
	}

	pub fn u64(value: u64) -> Value {
		Self::new(num::kind::Int::U64, num::Int { u64: value })
	}

	pub fn usize(value: usize) -> Value {
		Self::new(num::kind::Int::USize, num::Int { usize: value })
	}

	pub fn any(value: num::AnyInt) -> Value {
		Self::new(num::kind::Int::Any, num::Int { any: value })
	}
}

impl num::Float {
	pub fn new(typ: num::kind::Float, val: num::Float) -> Value {
		Value(Type::Float(typ), InnerValue { float: val })
	}

	pub fn f32(value: f32) -> Value {
		Self::new(num::kind::Float::F32, num::Float { f32: value })
	}

	pub fn f64(value: f64) -> Value {
		Self::new(num::kind::Float::F64, num::Float { f64: value })
	}

	pub fn any(value: f64) -> Value {
		Self::new(num::kind::Float::Any, num::Float { any: value })
	}
}
