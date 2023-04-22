use std::any::TypeId;

use super::util::*;
use super::*;

pub type AnyInt = u64;
pub type AnyFloat = f64;

#[derive(Copy, Clone)]
pub union Int {
	pub any: AnyInt,
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

impl Int {
	pub fn from<T: 'static>(value: T) -> Result<(kind::Int, Int), T> {
		when_type!(value: T =>
			i8 {
				return Ok((kind::Int::I8, Int{ i8: value }));
			}
			u8 {
				return Ok((kind::Int::U8, Int{ u8: value }));
			}
			i16 {
				return Ok((kind::Int::I16, Int{ i16: value }));
			}
			i32 {
				return Ok((kind::Int::I32, Int{ i32: value }));
			}
			i64 {
				return Ok((kind::Int::I64, Int{ i64: value }));
			}
			u16 {
				return Ok((kind::Int::U16, Int{ u16: value }));
			}
			u32 {
				return Ok((kind::Int::U32, Int{ u32: value }));
			}
			u64 {
				return Ok((kind::Int::U64, Int{ u64: value }));
			}
			isize {
				return Ok((kind::Int::ISize, Int{ isize: value }));
			}
			usize {
				return Ok((kind::Int::USize, Int{ usize: value }));
			}
		);
		Err(value)
	}

	pub fn get<T: 'static>(&self, kind: kind::Int) -> Option<&T> {
		match kind {
			kind::Int::I8 => {
				if TypeId::of::<T>() == TypeId::of::<i8>() {
					return Some(unsafe { std::mem::transmute(&self.i8) });
				}
			}
			kind::Int::U8 => {
				if TypeId::of::<T>() == TypeId::of::<u8>() {
					return Some(unsafe { std::mem::transmute(&self.u8) });
				}
			}

			kind::Int::I16 => {
				if TypeId::of::<T>() == TypeId::of::<i16>() {
					return Some(unsafe { std::mem::transmute(&self.i16) });
				}
			}
			kind::Int::U16 => {
				if TypeId::of::<T>() == TypeId::of::<u16>() {
					return Some(unsafe { std::mem::transmute(&self.u16) });
				}
			}

			kind::Int::I32 => {
				if TypeId::of::<T>() == TypeId::of::<i32>() {
					return Some(unsafe { std::mem::transmute(&self.i32) });
				}
			}
			kind::Int::U32 => {
				if TypeId::of::<T>() == TypeId::of::<u32>() {
					return Some(unsafe { std::mem::transmute(&self.u32) });
				}
			}

			kind::Int::I64 => {
				if TypeId::of::<T>() == TypeId::of::<i64>() {
					return Some(unsafe { std::mem::transmute(&self.i64) });
				}
			}
			kind::Int::U64 => {
				if TypeId::of::<T>() == TypeId::of::<u64>() {
					return Some(unsafe { std::mem::transmute(&self.u64) });
				}
			}

			kind::Int::ISize => {
				if TypeId::of::<T>() == TypeId::of::<isize>() {
					return Some(unsafe { std::mem::transmute(&self.isize) });
				}
			}
			kind::Int::USize => {
				if TypeId::of::<T>() == TypeId::of::<usize>() {
					return Some(unsafe { std::mem::transmute(&self.usize) });
				}
			}

			kind::Int::Any => {
				if TypeId::of::<T>() == TypeId::of::<AnyInt>() {
					return Some(unsafe { std::mem::transmute(&self.any) });
				}
			}
		}
		None
	}

	pub fn get_mut<T: 'static>(&mut self, kind: kind::Int) -> Option<&mut T> {
		match kind {
			kind::Int::I8 => {
				if TypeId::of::<T>() == TypeId::of::<i8>() {
					return Some(unsafe { std::mem::transmute(&mut self.i8) });
				}
			}
			kind::Int::U8 => {
				if TypeId::of::<T>() == TypeId::of::<u8>() {
					return Some(unsafe { std::mem::transmute(&mut self.u8) });
				}
			}

			kind::Int::I16 => {
				if TypeId::of::<T>() == TypeId::of::<i16>() {
					return Some(unsafe { std::mem::transmute(&mut self.i16) });
				}
			}
			kind::Int::U16 => {
				if TypeId::of::<T>() == TypeId::of::<u16>() {
					return Some(unsafe { std::mem::transmute(&mut self.u16) });
				}
			}

			kind::Int::I32 => {
				if TypeId::of::<T>() == TypeId::of::<i32>() {
					return Some(unsafe { std::mem::transmute(&mut self.i32) });
				}
			}
			kind::Int::U32 => {
				if TypeId::of::<T>() == TypeId::of::<u32>() {
					return Some(unsafe { std::mem::transmute(&mut self.u32) });
				}
			}

			kind::Int::I64 => {
				if TypeId::of::<T>() == TypeId::of::<i64>() {
					return Some(unsafe { std::mem::transmute(&mut self.i64) });
				}
			}
			kind::Int::U64 => {
				if TypeId::of::<T>() == TypeId::of::<u64>() {
					return Some(unsafe { std::mem::transmute(&mut self.u64) });
				}
			}

			kind::Int::ISize => {
				if TypeId::of::<T>() == TypeId::of::<isize>() {
					return Some(unsafe { std::mem::transmute(&mut self.isize) });
				}
			}
			kind::Int::USize => {
				if TypeId::of::<T>() == TypeId::of::<usize>() {
					return Some(unsafe { std::mem::transmute(&mut self.usize) });
				}
			}

			kind::Int::Any => {
				if TypeId::of::<T>() == TypeId::of::<AnyInt>() {
					return Some(unsafe { std::mem::transmute(&mut self.any) });
				}
			}
		}
		None
	}

	pub fn eq(&self, other: &Int, kind: kind::Int) -> bool {
		let res = unsafe {
			match kind {
				kind::Int::Any => self.any == other.any,
				kind::Int::I8 => self.i8 == other.i8,
				kind::Int::U8 => self.u8 == other.u8,
				kind::Int::I16 => self.i16 == other.i16,
				kind::Int::I32 => self.i32 == other.i32,
				kind::Int::I64 => self.i64 == other.i64,
				kind::Int::U16 => self.u16 == other.u16,
				kind::Int::U32 => self.u32 == other.u32,
				kind::Int::U64 => self.u64 == other.u64,
				kind::Int::ISize => self.isize == other.isize,
				kind::Int::USize => self.usize == other.usize,
			}
		};
		res
	}

	pub fn as_ref(&self, kind: kind::Int) -> &dyn IsValue {
		let res: &dyn IsValue = unsafe {
			match kind {
				kind::Int::Any => &self.any,
				kind::Int::I8 => &self.i8,
				kind::Int::U8 => &self.u8,
				kind::Int::I16 => &self.i16,
				kind::Int::I32 => &self.i32,
				kind::Int::I64 => &self.i64,
				kind::Int::U16 => &self.u16,
				kind::Int::U32 => &self.u32,
				kind::Int::U64 => &self.u64,
				kind::Int::ISize => &self.isize,
				kind::Int::USize => &self.usize,
			}
		};
		res
	}
}

#[derive(Copy, Clone)]
pub union Float {
	pub any: AnyFloat,
	pub f32: f32,
	pub f64: f64,
}

impl Float {
	pub fn from<T: 'static>(value: T) -> Result<(kind::Float, Float), T> {
		when_type!(value: T =>
			f32 {
				return Ok((kind::Float::F32, Float{ f32: value }));
			}
			f64 {
				return Ok((kind::Float::F64, Float{ f64: value }));
			}
		);
		Err(value)
	}

	pub fn get<T: 'static>(&self, kind: kind::Float) -> Option<&T> {
		match kind {
			kind::Float::F32 => {
				if TypeId::of::<T>() == TypeId::of::<f32>() {
					return Some(unsafe { std::mem::transmute(&self.f32) });
				}
			}
			kind::Float::F64 => {
				if TypeId::of::<T>() == TypeId::of::<f64>() {
					return Some(unsafe { std::mem::transmute(&self.f64) });
				}
			}
			kind::Float::Any => {
				if TypeId::of::<T>() == TypeId::of::<AnyFloat>() {
					return Some(unsafe { std::mem::transmute(&self.any) });
				}
			}
		}
		None
	}

	pub fn get_mut<T: 'static>(&mut self, kind: kind::Float) -> Option<&mut T> {
		match kind {
			kind::Float::F32 => {
				if TypeId::of::<T>() == TypeId::of::<f32>() {
					return Some(unsafe { std::mem::transmute(&mut self.f32) });
				}
			}
			kind::Float::F64 => {
				if TypeId::of::<T>() == TypeId::of::<f64>() {
					return Some(unsafe { std::mem::transmute(&mut self.f64) });
				}
			}
			kind::Float::Any => {
				if TypeId::of::<T>() == TypeId::of::<AnyFloat>() {
					return Some(unsafe { std::mem::transmute(&mut self.any) });
				}
			}
		}
		None
	}

	pub fn eq(&self, other: &Float, kind: kind::Float) -> bool {
		let res = unsafe {
			match kind {
				kind::Float::Any => self.any == other.any,
				kind::Float::F32 => self.f32 == other.f32,
				kind::Float::F64 => self.f64 == other.f64,
			}
		};
		res
	}

	pub fn as_ref(&self, kind: kind::Float) -> &dyn IsValue {
		let res: &dyn IsValue = unsafe {
			match kind {
				kind::Float::Any => &self.any,
				kind::Float::F32 => &self.f32,
				kind::Float::F64 => &self.f64,
			}
		};
		res
	}
}

pub mod kind {
	#[derive(Copy, Clone, Eq, PartialEq, Debug)]
	pub enum Int {
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

	#[derive(Copy, Clone, Eq, PartialEq, Debug)]
	pub enum Float {
		Any,
		F32,
		F64,
	}
}
