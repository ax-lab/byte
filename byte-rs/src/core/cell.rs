use std::any::Any;
use std::any::TypeId;
use std::fmt::*;
use std::ops::Deref;
use std::ops::DerefMut;
use std::panic::UnwindSafe;
use std::ptr::NonNull;
use std::sync::Arc;
use std::sync::RwLock;
use std::sync::RwLockReadGuard;
use std::sync::RwLockWriteGuard;

use super::num;
use super::util::*;

/// Trait for values that can be used in a [`Cell`] and other dynamic contexts.
///
/// This trait provides a blanket implementation for all supported values.
pub trait IsValue: CanBox + DynClone {}

impl<T: CanBox + DynClone> IsValue for T {}

pub trait CanBox: Any + Send + Sync + UnwindSafe {}

impl<T: Any + Send + Sync + UnwindSafe> CanBox for T {}

/// Blanket trait providing dynamic cloning capabilities for types that
/// implement the [`Clone`] trait.
pub trait DynClone {
	fn clone_box(&self) -> Box<dyn IsValue>;
}

impl<T: CanBox + Clone> DynClone for T {
	fn clone_box(&self) -> Box<dyn IsValue> {
		Box::new(self.clone())
	}
}

/// Cell that can store any kind of value and provide dynamic type binding
/// to that value.
pub struct Cell {
	kind: CellKind,
	data: CellData,
}

impl Debug for Cell {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result {
		todo!()
	}
}

impl Clone for Cell {
	fn clone(&self) -> Self {
		if let CellKind::Other(..) = self.kind {
			unsafe {
				Arc::increment_strong_count(self.data.other.ptr);
			};
		}
		Cell {
			kind: self.kind,
			data: self.data,
		}
	}
}

impl Drop for Cell {
	fn drop(&mut self) {
		if let CellKind::Other(..) = self.kind {
			unsafe {
				Arc::decrement_strong_count(self.data.other.ptr);
			};
		}
	}
}

impl Cell {
	pub fn unit() -> Cell {
		Cell {
			kind: CellKind::Unit,
			data: unsafe { std::mem::zeroed() },
		}
	}

	pub fn never() -> Cell {
		Cell {
			kind: CellKind::Never,
			data: unsafe { std::mem::zeroed() },
		}
	}

	pub fn any_int(value: num::AnyInt) -> Cell {
		Cell {
			kind: CellKind::Int(num::kind::Int::Any),
			data: CellData {
				int: num::Int { any: value },
			},
		}
	}

	pub fn any_float(value: num::AnyFloat) -> Cell {
		Cell {
			kind: CellKind::Float(num::kind::Float::Any),
			data: CellData {
				float: num::Float { any: value },
			},
		}
	}

	pub fn from<T: CanBox>(value: T) -> Cell {
		when_type!(value: T =>
			bool {
				return Cell {
					kind: CellKind::Bool,
					data: CellData { bool: value },
				}
			}

			Cell {
				return value;
			}
		);

		let value = match num::Int::from(value) {
			Ok((kind, int)) => {
				return Cell {
					kind: CellKind::Int(kind),
					data: CellData { int },
				}
			}
			Err(value) => value,
		};

		let value = match num::Float::from(value) {
			Ok((kind, float)) => {
				return Cell {
					kind: CellKind::Float(kind),
					data: CellData { float },
				}
			}
			Err(value) => value,
		};

		let other = CellPtr::new(value);
		Cell {
			kind: CellKind::Other(TypeId::of::<T>()),
			data: CellData { other },
		}
	}

	pub fn is_unit(&self) -> bool {
		self.kind == CellKind::Unit
	}

	pub fn is_never(&self) -> bool {
		self.kind == CellKind::Never
	}

	pub fn get<T: CanBox>(&self) -> Option<Ref<T>> {
		match self.kind {
			CellKind::Never | CellKind::Unit => None,
			CellKind::Bool => {
				if TypeId::of::<T>() == TypeId::of::<bool>() {
					let ptr = unsafe { std::mem::transmute(&self.data.bool) };
					Some(Ref::Plain(ptr))
				} else {
					None
				}
			}
			CellKind::Int(kind) => {
				let ptr = unsafe { self.data.int.get::<T>(kind) };
				ptr.map(|x| Ref::Plain(x))
			}
			CellKind::Float(kind) => {
				let ptr = unsafe { self.data.float.get::<T>(kind) };
				ptr.map(|x| Ref::Plain(x))
			}
			CellKind::Other(id) => {
				if id == TypeId::of::<T>() {
					let ptr = unsafe { self.data.other.ptr.as_ref().unwrap() };
					let ptr = ptr.read().unwrap();
					Some(Ref::Boxed(unsafe { std::mem::transmute(ptr) }))
				} else {
					None
				}
			}
		}
	}

	pub fn get_mut<T: CanBox>(&mut self) -> Option<RefMut<T>> {
		match self.kind {
			CellKind::Never | CellKind::Unit => None,
			CellKind::Bool => {
				if TypeId::of::<T>() == TypeId::of::<bool>() {
					let ptr = unsafe { std::mem::transmute(&mut self.data.bool) };
					Some(RefMut::Plain(ptr))
				} else {
					None
				}
			}
			CellKind::Int(kind) => {
				let ptr = unsafe { self.data.int.get_mut::<T>(kind) };
				ptr.map(|x| RefMut::Plain(x))
			}
			CellKind::Float(kind) => {
				let ptr = unsafe { self.data.float.get_mut::<T>(kind) };
				ptr.map(|x| RefMut::Plain(x))
			}
			CellKind::Other(id) => {
				if id == TypeId::of::<T>() {
					let ptr = unsafe { self.data.other.ptr.as_ref().unwrap() };
					let mut ptr = ptr.write().unwrap();
					Some(RefMut::Boxed(unsafe { std::mem::transmute(ptr) }))
				} else {
					None
				}
			}
		}
	}
}

pub enum Ref<'a, T: CanBox> {
	Plain(&'a T),
	Boxed(RwLockReadGuard<'a, Box<T>>),
}

pub enum RefMut<'a, T: CanBox> {
	Plain(&'a mut T),
	Boxed(RwLockWriteGuard<'a, Box<T>>),
}

impl<'a, T: CanBox> Deref for Ref<'a, T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		match self {
			Ref::Plain(ptr) => ptr,
			Ref::Boxed(ptr) => ptr,
		}
	}
}

impl<'a, T: CanBox> Deref for RefMut<'a, T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		match self {
			RefMut::Plain(ptr) => ptr,
			RefMut::Boxed(ptr) => ptr,
		}
	}
}

impl<'a, T: CanBox> DerefMut for RefMut<'a, T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		match self {
			RefMut::Plain(ptr) => ptr,
			RefMut::Boxed(ptr) => ptr,
		}
	}
}

#[repr(C)]
#[derive(Copy, Clone)]
union CellData {
	bool: bool,
	int: num::Int,
	float: num::Float,
	other: CellPtr,
}

#[derive(Copy, Clone)]
struct CellPtr {
	ptr: *const RwLock<Box<dyn CanBox>>,
}

unsafe impl Send for CellPtr {}
unsafe impl Sync for CellPtr {}

impl CellPtr {
	pub fn new<T: CanBox>(value: T) -> Self {
		let ptr: Box<dyn CanBox> = Box::new(value);
		let ptr = Arc::new(RwLock::new(ptr));
		CellPtr {
			ptr: Arc::into_raw(ptr),
		}
	}
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum CellKind {
	Unit,
	Never,
	Bool,
	Int(num::kind::Int),
	Float(num::kind::Float),
	Other(TypeId),
}

const _: () = {
	fn assert<T: IsValue>() {}

	fn assert_all() {
		assert::<Cell>();
		assert::<String>();
		assert::<u32>();
	}
};

#[cfg(test)]
mod tests {

	use super::*;

	#[test]
	fn simple_types() {
		// unit
		let unit = Cell::unit();
		assert!(unit.is_unit());

		// never
		let never = Cell::never();
		assert!(never.is_never());

		// bool
		let mut bool = Cell::from(true);
		check(&bool, true, false);

		let bool = Cell::from(false);
		check(&bool, false, true);

		// i8 and u8
		let int = Cell::from(42i8);
		check(&int, 42i8, 123);

		let int = Cell::from(42u8);
		check(&int, 42u8, 123);

		// i16 and u16
		let int = Cell::from(42i16);
		check(&int, 42i16, 123);

		let int = Cell::from(42u16);
		check(&int, 42u16, 123);

		// i32 and u32
		let int = Cell::from(42i32);
		check(&int, 42i32, 123);

		let int = Cell::from(42u32);
		check(&int, 42u32, 123);

		// i64 and u64
		let int = Cell::from(42i64);
		check(&int, 42i64, 123);

		let int = Cell::from(42u64);
		check(&int, 42u64, 123);

		// isize and usize
		let int = Cell::from(42isize);
		check(&int, 42isize, 123);

		let int = Cell::from(42usize);
		check(&int, 42usize, 123);

		// floats
		let float = Cell::from(42.0f32);
		check(&float, 42.0f32, 123.0);

		let float = Cell::from(42.0f64);
		check(&float, 42.0f64, 123.0);

		// any
		let v1: num::AnyInt = 42;
		let v2: num::AnyInt = 123;
		let any = Cell::any_int(v1);
		check(&any, v1, v2);

		// any
		let v1: num::AnyFloat = 42.0;
		let v2: num::AnyFloat = 123.0;
		let any = Cell::any_float(v1);
		check(&any, v1, v2);

		fn check<T: IsValue + PartialEq + Clone + Debug>(cell: &Cell, v1: T, v2: T) {
			assert!(cell.get::<String>().is_none());
			assert_eq!(*cell.get::<T>().unwrap(), v1);

			let mut cell = cell.clone();
			assert!(cell.get_mut::<String>().is_none());
			assert_eq!(*cell.get::<T>().unwrap(), v1);
			assert_eq!(*cell.get_mut::<T>().unwrap(), v1);

			*(cell.get_mut::<T>().unwrap()) = v2.clone();
			assert_eq!(*cell.get::<T>().unwrap(), v2);
			assert_eq!(*cell.get_mut::<T>().unwrap(), v2);
		}
	}

	#[test]
	fn complex_type() {
		struct X {
			data: Arc<RwLock<i32>>,
		}

		let my_data = Arc::new(RwLock::new(42));
		let my_type = X {
			data: my_data.clone(),
		};

		let mut cell = Cell::from(my_type);
		assert!(cell.get::<String>().is_none());
		assert!(cell.get_mut::<String>().is_none());

		let value = cell.get::<X>().unwrap();
		assert_eq!(*value.data.read().unwrap(), 42);
		drop(value);

		let value = cell.get_mut::<X>().unwrap();
		assert_eq!(*value.data.read().unwrap(), 42);
		*value.data.write().unwrap() = 123;

		assert_eq!(*value.data.read().unwrap(), 123);
		drop(value);

		assert_eq!(*my_data.read().unwrap(), 123);
	}
}
