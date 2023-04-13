use std::any::{Any, TypeId};

use super::*;

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

	pub fn as_ref<T: 'static>(&self) -> Option<&T> {
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

	pub fn as_mut<T: 'static>(&mut self) -> Option<&mut T> {
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
	pub int: ValueInt,
	pub float: ValueFloat,
	pub ptr: *mut (),
}

impl Default for InnerValue {
	fn default() -> Self {
		InnerValue {
			int: ValueInt { u64: 0 },
		}
	}
}

impl InnerValue {
	pub fn pack<T: 'static>(value: T) -> InnerValue {
		let ptr = Box::new(value);
		let ptr = Box::into_raw(ptr) as *mut ();
		InnerValue { ptr }
	}

	pub unsafe fn unpack<T: 'static>(self) -> T {
		let ptr = self.ptr as *mut T;
		let ptr = Box::from_raw(ptr);
		*ptr
	}

	pub unsafe fn as_ref<T: 'static>(&self) -> Option<&T> {
		let ptr = self.ptr as *const T;
		ptr.as_ref()
	}

	pub unsafe fn as_mut<T: 'static>(&mut self) -> Option<&mut T> {
		let ptr = self.ptr as *mut T;
		ptr.as_mut()
	}
}

#[derive(Copy, Clone)]
pub union ValueInt {
	pub any: u64,
	pub i8: i8,
	pub u8: u8,
	pub i16: i16,
	pub i32: i32,
	pub i64: i64,
	pub u16: u16,
	pub u32: u32,
	pub u64: u64,
	pub isize: isize,
	pub usize: usize,
}

impl ValueInt {
	pub fn new(typ: TypeInt, val: ValueInt) -> Value {
		Value(Type::Int(typ), InnerValue { int: val })
	}

	pub fn i8(value: i8) -> Value {
		Self::new(TypeInt::I8, ValueInt { i8: value })
	}

	pub fn i16(value: i16) -> Value {
		Self::new(TypeInt::I16, ValueInt { i16: value })
	}

	pub fn i32(value: i32) -> Value {
		Self::new(TypeInt::I32, ValueInt { i32: value })
	}

	pub fn i64(value: i64) -> Value {
		Self::new(TypeInt::I64, ValueInt { i64: value })
	}

	pub fn isize(value: isize) -> Value {
		Self::new(TypeInt::ISize, ValueInt { isize: value })
	}

	pub fn u8(value: u8) -> Value {
		Self::new(TypeInt::U8, ValueInt { u8: value })
	}

	pub fn u16(value: u16) -> Value {
		Self::new(TypeInt::U16, ValueInt { u16: value })
	}

	pub fn u32(value: u32) -> Value {
		Self::new(TypeInt::U32, ValueInt { u32: value })
	}

	pub fn u64(value: u64) -> Value {
		Self::new(TypeInt::U64, ValueInt { u64: value })
	}

	pub fn usize(value: usize) -> Value {
		Self::new(TypeInt::USize, ValueInt { usize: value })
	}

	pub fn any(value: u64) -> Value {
		Self::new(TypeInt::Any, ValueInt { any: value })
	}
}

#[derive(Copy, Clone)]
pub union ValueFloat {
	pub any: f64,
	pub f32: f32,
	pub f64: f64,
}

impl ValueFloat {
	pub fn new(typ: TypeFloat, val: ValueFloat) -> Value {
		Value(Type::Float(typ), InnerValue { float: val })
	}

	pub fn f32(value: f32) -> Value {
		Self::new(TypeFloat::F32, ValueFloat { f32: value })
	}

	pub fn f64(value: f64) -> Value {
		Self::new(TypeFloat::F64, ValueFloat { f64: value })
	}

	pub fn any(value: f64) -> Value {
		Self::new(TypeFloat::Any, ValueFloat { any: value })
	}
}
