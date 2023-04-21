use std::any::Any;
use std::any::TypeId;
use std::fmt::*;
use std::panic::UnwindSafe;
use std::sync::Arc;

use super::num;
use super::util::*;

/// Trait for any generic value that can be used with a [`Cell`].
///
/// This trait provides a blanket implementation for all supported values.
pub trait IsCellValue: CanBox + DynClone + DynEq {}

impl<T: CanBox + DynClone + DynEq> IsCellValue for T {}

pub trait CanBox: Any + Send + Sync + UnwindSafe {}

impl<T: Any + Send + Sync + UnwindSafe> CanBox for T {}

/// Dynamic clone trait. Provides a blanket implementation for [`Clone`] types.
pub trait DynClone {
	fn clone_box(&self) -> Arc<dyn IsCellValue>;
}

impl<T: CanBox + Clone + DynEq> DynClone for T {
	fn clone_box(&self) -> Arc<dyn IsCellValue> {
		Arc::new(self.clone())
	}
}

/// Dynamic equality trait. Provides a blanket implementation for [`Eq`] types.
pub trait DynEq {
	fn is_eq(&self, other: &Cell) -> bool;
}

impl<T: CanBox + PartialEq> DynEq for T {
	fn is_eq(&self, other: &Cell) -> bool {
		if let Some(other) = other.get::<T>() {
			self == other
		} else {
			false
		}
	}
}

/// Stores a dynamically typed value with [`Arc`] style sharing and copy
/// on write semantics.
pub struct Cell {
	kind: CellKind,
	data: CellData,
}

#[allow(unused)]
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

	pub fn from<T: IsCellValue>(value: T) -> Cell {
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

			&str {
				return Cell {
					kind: CellKind::Str,
					data: CellData { str: value }
				}
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
			kind: CellKind::Other,
			data: CellData { other },
		}
	}

	pub fn kind(&self) -> CellKind {
		self.kind
	}

	pub fn is_unit(&self) -> bool {
		self.kind == CellKind::Unit
	}

	pub fn is_never(&self) -> bool {
		self.kind == CellKind::Never
	}

	pub fn is_int(&self) -> bool {
		matches!(self.kind, CellKind::Int(..))
	}

	pub fn is_float(&self) -> bool {
		matches!(self.kind, CellKind::Float(..))
	}

	pub fn is_any_int(&self) -> bool {
		self.kind == CellKind::Int(num::kind::Int::Any)
	}

	pub fn is_any_float(&self) -> bool {
		self.kind == CellKind::Float(num::kind::Float::Any)
	}

	pub fn get<T: CanBox>(&self) -> Option<&T> {
		match self.kind {
			CellKind::Never | CellKind::Unit => None,
			CellKind::Bool => {
				if TypeId::of::<T>() == TypeId::of::<bool>() {
					let ptr = unsafe { std::mem::transmute(&self.data.bool) };
					Some(ptr)
				} else {
					None
				}
			}
			CellKind::Str => {
				if TypeId::of::<T>() == TypeId::of::<&str>() {
					let ptr = unsafe { std::mem::transmute(&self.data.str) };
					Some(ptr)
				} else {
					None
				}
			}
			CellKind::Int(kind) => unsafe { self.data.int.get::<T>(kind) },
			CellKind::Float(kind) => unsafe { self.data.float.get::<T>(kind) },
			CellKind::Other => {
				let data = unsafe { self.data.other };
				if data.id() == TypeId::of::<T>() {
					let ptr = unsafe {
						let ptr = data.ptr as *const T;
						ptr.as_ref().unwrap()
					};
					Some(ptr)
				} else {
					None
				}
			}
		}
	}

	pub fn get_mut<T: CanBox>(&mut self) -> Option<&mut T> {
		match self.kind {
			CellKind::Never | CellKind::Unit => None,
			CellKind::Bool => {
				if TypeId::of::<T>() == TypeId::of::<bool>() {
					let ptr = unsafe { std::mem::transmute(&mut self.data.bool) };
					Some(ptr)
				} else {
					None
				}
			}
			CellKind::Str => {
				if TypeId::of::<T>() == TypeId::of::<&str>() {
					let ptr = unsafe { std::mem::transmute(&mut self.data.str) };
					Some(ptr)
				} else {
					None
				}
			}
			CellKind::Int(kind) => unsafe { self.data.int.get_mut::<T>(kind) },
			CellKind::Float(kind) => unsafe { self.data.float.get_mut::<T>(kind) },
			CellKind::Other => {
				let mut data = unsafe { &mut self.data.other };
				if data.id() == TypeId::of::<T>() {
					let ptr = unsafe { (data.get_mut() as *mut T).as_mut().unwrap() };
					Some(ptr)
				} else {
					None
				}
			}
		}
	}

	pub fn as_str(&self) -> Option<&str> {
		match self.kind {
			CellKind::Str => unsafe { Some(self.data.str) },
			CellKind::Other => self.get::<String>().map(|x| x.as_str()),
			_ => None,
		}
	}
}

//----------------------------------------------------------------------------//
// Cell traits
//----------------------------------------------------------------------------//

impl Clone for Cell {
	fn clone(&self) -> Self {
		if let CellKind::Other = self.kind {
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
		if let CellKind::Other = self.kind {
			unsafe {
				Arc::decrement_strong_count(self.data.other.ptr);
			};
		}
	}
}

impl PartialEq for Cell {
	fn eq(&self, other: &Self) -> bool {
		if self.kind == CellKind::Str || other.kind == CellKind::Str {
			self.as_str() == other.as_str()
		} else if self.kind != other.kind && other.kind == CellKind::Other {
			other.eq(self)
		} else {
			match self.kind {
				CellKind::Unit => other.kind == self.kind,
				CellKind::Never => other.kind == self.kind,
				CellKind::Bool => {
					other.kind == self.kind && unsafe { self.data.bool == other.data.bool }
				}
				CellKind::Str => self.as_str() == other.as_str(),
				CellKind::Int(kind) => {
					other.kind == self.kind && unsafe { self.data.int.eq(&other.data.int, kind) }
				}
				CellKind::Float(kind) => {
					other.kind == self.kind
						&& unsafe { self.data.float.eq(&other.data.float, kind) }
				}
				CellKind::Other => {
					let ptr = unsafe { self.data.other };
					let ptr = ptr.as_ref();
					ptr.is_eq(other)
				}
			}
		}
	}
}

//----------------------------------------------------------------------------//
// Utility types
//----------------------------------------------------------------------------//

#[repr(C)]
#[derive(Copy, Clone)]
union CellData {
	bool: bool,
	int: num::Int,
	float: num::Float,
	str: &'static str,
	other: CellPtr,
}

#[derive(Copy, Clone)]
struct CellPtr {
	id: TypeId,
	ptr: *const dyn IsCellValue,
}

unsafe impl Send for CellPtr {}
unsafe impl Sync for CellPtr {}

impl UnwindSafe for CellPtr {}

impl CellPtr {
	pub fn new<T: IsCellValue>(value: T) -> Self {
		let ptr: Arc<dyn IsCellValue> = Arc::new(value);
		CellPtr {
			id: TypeId::of::<T>(),
			ptr: Arc::into_raw(ptr),
		}
	}

	pub fn id(&self) -> TypeId {
		self.id
	}

	pub fn as_ref(&self) -> &dyn IsCellValue {
		unsafe { self.ptr.as_ref() }.unwrap()
	}

	pub fn get_mut(&mut self) -> *mut dyn IsCellValue {
		unsafe {
			let arc = std::mem::ManuallyDrop::new(Arc::from_raw(self.ptr));
			let mut arc = if Arc::strong_count(&arc) != 1 {
				let new_arc = arc.clone_box();
				Arc::decrement_strong_count(self.ptr);
				self.ptr = Arc::into_raw(new_arc);
				std::mem::ManuallyDrop::new(Arc::from_raw(self.ptr))
			} else {
				arc
			};
			let ptr = Arc::get_mut(&mut arc).unwrap();
			ptr
		}
	}
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum CellKind {
	Unit,
	Never,
	Bool,
	Str,
	Int(num::kind::Int),
	Float(num::kind::Float),
	Other,
}

//----------------------------------------------------------------------------//
// Tests & assertions
//----------------------------------------------------------------------------//

const _: () = {
	fn assert<T: IsCellValue>() {}

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
		let bool = Cell::from(true);
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
		assert!(any.is_any_int());
		check(&any, v1, v2);

		// any
		let v1: num::AnyFloat = 42.0;
		let v2: num::AnyFloat = 123.0;
		let any = Cell::any_float(v1);
		assert!(any.is_any_float());
		check(&any, v1, v2);

		fn check<T: IsCellValue + PartialEq + Clone + Debug>(cell: &Cell, v1: T, v2: T) {
			// check that we don't cast to the wrong type
			assert!(cell.get::<String>().is_none());

			// make sure we are able to get the value
			assert_eq!(*cell.get::<T>().unwrap(), v1);

			// make sure invariants hold after cloning
			let mut cell = cell.clone();
			assert!(cell.get_mut::<String>().is_none());
			assert_eq!(*cell.get::<T>().unwrap(), v1);

			// test the mutable reference as well
			assert_eq!(*cell.get_mut::<T>().unwrap(), v1);

			let saved = cell.clone();

			// change the value and check that it applied correctly
			*(cell.get_mut::<T>().unwrap()) = v2.clone();
			assert_eq!(*cell.get::<T>().unwrap(), v2);
			assert_eq!(*cell.get_mut::<T>().unwrap(), v2);

			// make sure the clone does not change
			assert_eq!(*saved.get::<T>().unwrap(), v1);

			// make sure a cell is equal to its clone
			assert!(cell == cell.clone());
			assert!(cell != saved);
		}
	}

	#[test]
	fn complex_type() {
		use std::sync::atomic::*;

		// Number of instances of X. Used to test drop and copy on write are
		// correct.
		static COUNTER: AtomicUsize = AtomicUsize::new(0);
		let count = || COUNTER.load(Ordering::SeqCst) as usize;

		struct X<'a> {
			data: u32,
			cnt: &'a AtomicUsize,
		}

		impl<'a> PartialEq for X<'a> {
			fn eq(&self, other: &Self) -> bool {
				self.data == other.data
			}
		}

		impl<'a> X<'a> {
			pub fn new(data: u32, cnt: &'a AtomicUsize) -> Self {
				cnt.fetch_add(1, Ordering::SeqCst);
				Self { data, cnt }
			}
		}

		impl<'a> Clone for X<'a> {
			fn clone(&self) -> Self {
				self.cnt.fetch_add(1, Ordering::SeqCst);
				Self {
					data: self.data.clone(),
					cnt: self.cnt.clone(),
				}
			}
		}

		impl<'a> Drop for X<'a> {
			fn drop(&mut self) {
				self.cnt.fetch_sub(1, Ordering::SeqCst);
			}
		}

		// Create a new value and cell.
		let x = X::new(42, &COUNTER);
		let mut cell = Cell::from(x);

		// Make sure we are not able to cast to the wrong type.
		assert!(cell.get::<String>().is_none());
		assert!(cell.get_mut::<String>().is_none());

		// So far we only created one instance
		assert_eq!(count(), 1);

		let value = cell.get_mut::<X>().unwrap();
		assert_eq!(count(), 1); // we are the single instance, so no copy
		drop(value);

		// Save the current value for later
		let saved = cell.clone();
		assert_eq!(count(), 1); // clone is copy-on-write

		// Test equality
		assert!(cell == saved);

		// Check that we can get a reference to the value
		let value = cell.get::<X>().unwrap();
		assert_eq!(value.data, 42);
		drop(value);

		// Check that we can get a mutable reference, this will copy
		let value = cell.get_mut::<X>().unwrap();
		assert_eq!(count(), 2);
		assert_eq!(value.data, 42);

		// Change the value
		value.data = 123;
		drop(value);

		assert!(cell != saved);

		// Check that the change affected the original...
		let value = cell.get::<X>().unwrap();
		assert_eq!(value.data, 123);
		drop(value);

		// ...but not the clone
		assert_eq!(saved.get::<X>().unwrap().data, 42);

		// At the end we must have two instances...
		assert_eq!(count(), 2);
		drop(saved);
		drop(cell);

		// ...both must be properly dropped at the end
		assert_eq!(count(), 0);
	}

	#[test]
	fn strings() {
		let a = Cell::from("abc");
		let b = Cell::from("abc");
		let c = Cell::from("123");

		// make sure we are using the static str kind
		assert_eq!(a.kind(), CellKind::Str);
		assert_eq!(b.kind(), CellKind::Str);
		assert_eq!(c.kind(), CellKind::Str);

		// test retrieving values directly
		assert!(a.get::<&str>() == Some(&"abc"));
		assert!(b.clone().get::<&str>() == Some(&"abc"));
		assert!(c.get::<&str>() == Some(&"123"));

		// test comparison
		assert!(a == b);
		assert!(a != c);

		// test as_str
		assert_eq!(a.as_str(), Some("abc"));
		assert_eq!(b.as_str(), Some("abc"));
		assert_eq!(c.as_str(), Some("123"));

		// test owned strings
		let a = Cell::from(String::from("abc"));
		let b = Cell::from(String::from("abc"));
		let c = Cell::from(String::from("123"));

		assert!(a == b);
		assert!(a != c);

		assert_eq!(a.as_str(), Some("abc"));
		assert_eq!(b.as_str(), Some("abc"));
		assert_eq!(c.as_str(), Some("123"));

		// test comparison between different string types
		assert!(Cell::from("123") == Cell::from(String::from("123")));
		assert!(Cell::from(String::from("123")) == Cell::from("123"));

		// test with non-string types
		let x = Cell::from(123);
		assert!(x.as_str() == None);
	}
}
