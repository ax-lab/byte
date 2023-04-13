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
				if typ.type_id() == TypeId::of::<T>() {
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
	pub ptr: *const (),
}

impl Default for InnerValue {
	fn default() -> Self {
		InnerValue {
			int: ValueInt { u64: 0 },
		}
	}
}

impl InnerValue {
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
	pub any: usize,
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

#[derive(Copy, Clone)]
pub union ValueFloat {
	pub any: f64,
	pub f32: f32,
	pub f64: f64,
}
