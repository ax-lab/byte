use std::{
	any::{Any, TypeId},
	fmt::{Debug, Display},
	sync::Arc,
};

/// Extension to [`std::any::Any`] providing dynamic typing and the basic
/// requirements for a generic value.
///
/// As with [`Any`], this trait provides a blanket implementation for all
/// supported types.
///
/// Types must be [`Send`] + [`Sync`], implement [`std::fmt::Debug`], and
/// must not contain non-`'static` references.
pub trait IsValue: Any + Send + Sync + Debug {}

impl<T: Any + Send + Sync + Debug> IsValue for T {}

/// Provides dynamic typing with traits.
///
/// To implement this trait use the [`has_traits`] macro. To retrieve a trait
/// from a [`HasTraits`] value use the [`to_trait`] macro.
pub trait HasTraits: IsValue {
	fn get_trait(&self, type_id: TypeId) -> Option<&dyn HasTraits> {
		let _ = type_id;
		None
	}
}

mod macros {
	#[allow(unused)]
	use super::HasTraits;

	/// Implements the [`HasTraits`] macros for a type, expanding to an `impl`
	/// block.
	///
	/// # Examples
	///
	/// ```
	/// struct SomeType {}
	///
	/// trait A {
	///     fn a(&self) {}
	/// }
	///
	/// trait B {
	///     fn b(&self) {}
	/// }
	///
	/// has_traits!(SomeType, A, B);
	///
	/// impl A for SomeType {}
	/// impl B for SomeType {}
	///
	/// fn somewhere(value: &dyn IsValue) {
	///     let value = to_trait!(value, A).unwrap();
	///     value.a();
	/// }
	/// ```
	#[macro_export]
	macro_rules! has_traits {
		($me:path $( : $($typ:path),+ )?) => {
			impl crate::core::any::HasTraits for $me {
				fn get_trait(
					&self,
					type_id: ::std::any::TypeId,
				) -> Option<&dyn crate::core::any::HasTraits> {
					let _ = type_id;
					$($(
						if (type_id == ::std::any::TypeId::of::<dyn $typ>()) {
							unsafe {
								let me = self as &dyn $typ;
								let me = std::mem::transmute(me);
								return Some(me);
							}
						}
					)+)?
					None
				}
			}
		};
	}

	#[macro_export]
	macro_rules! to_trait {
		($me:expr, $target:path) => {{
			let me = $me;
			let id = ::std::any::TypeId::of::<dyn $target>();
			let me = crate::core::any::HasTraits::get_trait(me, id);
			if let Some(me) = me {
				unsafe {
					let me: &dyn $target = std::mem::transmute(me);
					Some(me)
				}
			} else {
				None
			}
		}};
	}

	pub use has_traits;
	pub use to_trait;
}

pub use macros::has_traits;
pub use macros::to_trait;

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_has_traits() {
		let v = Arc::new(SomeType {
			a: "from A".into(),
			b: "from B".into(),
		});
		assert_eq!(v.say(), "from some type");

		let va = to_trait!(v.as_ref(), A).unwrap();
		let vb = to_trait!(v.as_ref(), B).unwrap();
		assert_eq!(va.a(), "from A");
		assert_eq!(vb.b(), "from B");

		let v = v.as_ref();
		assert!(to_trait!(v, C).is_none());
	}

	has_traits!(SomeType: A, B);

	trait A {
		fn a(&self) -> String;
	}

	trait B {
		fn b(&self) -> String;
	}

	trait C {
		fn c(&self) -> String;
	}

	#[derive(Debug)]
	struct SomeType {
		a: String,
		b: String,
	}

	impl SomeType {
		fn say(&self) -> String {
			format!("from some type")
		}
	}

	impl A for SomeType {
		fn a(&self) -> String {
			self.a.clone()
		}
	}

	impl B for SomeType {
		fn b(&self) -> String {
			self.b.clone()
		}
	}
}

//----------------------------------------------------------------------------//
// OLD CODE [deprecated]
//----------------------------------------------------------------------------//

/// This trait provides the minimum features required for a [`Value`].
///
/// To implement this for `Display + Debug + Eq` types see [`is_value`].
pub trait IsAnyValue: Debug + 'static + Send + Sync {
	fn output(&self, f: &mut std::fmt::Formatter, debug: bool) -> std::fmt::Result;

	fn is_eq(&self, other: &Value) -> bool;
}

/// Provides ref-counted storage for a dynamically typed value.
#[derive(Clone)]
pub struct Value {
	type_id: TypeId,
	value: Arc<dyn IsAnyValue>,
}

impl Value {
	pub fn new<T: IsAnyValue>(value: T) -> Value {
		Value {
			type_id: TypeId::of::<T>(),
			value: Arc::new(value),
		}
	}

	pub fn get<T: 'static>(&self) -> Option<&T> {
		if self.type_id == TypeId::of::<T>() {
			let ptr = self.value.as_ref();
			let ptr = unsafe { &*(ptr as *const dyn IsAnyValue as *const T) };
			Some(ptr)
		} else {
			None
		}
	}
}

/// Implement the [`IsValue`] trait for types that implement [`Display`],
/// [`Debug`], and [`Eq`].
macro_rules! is_value {
	($t:ty) => {
		impl IsAnyValue for $t {
			fn output(&self, f: &mut std::fmt::Formatter, debug: bool) -> std::fmt::Result {
				if debug {
					write!(f, "{}:={self:?}", stringify!($t))
				} else {
					write!(f, "{self}")
				}
			}

			fn is_eq(&self, other: &Value) -> bool {
				if let Some(other) = other.get::<Self>() {
					self == other
				} else {
					false
				}
			}
		}
	};

	($t:ty, debug) => {
		impl IsAnyValue for $t {
			fn output(&self, f: &mut std::fmt::Formatter, debug: bool) -> std::fmt::Result {
				if debug {
					write!(f, "{}<{self:?}>", stringify!($t))
				} else {
					write!(f, "{self:?}")
				}
			}

			fn is_eq(&self, other: &Value) -> bool {
				if let Some(other) = other.get::<Self>() {
					self == other
				} else {
					false
				}
			}
		}
	};
}

pub(crate) use is_value;

is_value!(&'static str);
is_value!(String);
is_value!(bool);
is_value!((), debug);

is_value!(i8);
is_value!(i16);
is_value!(i32);
is_value!(i64);
is_value!(i128);
is_value!(isize);

is_value!(u8);
is_value!(u16);
is_value!(u32);
is_value!(u64);
is_value!(u128);
is_value!(usize);

impl Display for Value {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.value.output(f, false)
	}
}

impl Debug for Value {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.value.output(f, true)
	}
}

impl PartialEq for Value {
	fn eq(&self, other: &Self) -> bool {
		self.value.is_eq(other)
	}
}

impl Eq for Value {}

#[cfg(test)]
mod tests_old {
	use super::*;

	#[test]
	fn any_value() {
		let a = Value::new("abc".to_string());
		let b = Value::new("123".to_string());
		let c = Value::new(123);

		assert_eq!(a.get::<String>(), Some(&"abc".into()));
		assert_eq!(b.get::<String>(), Some(&"123".into()));
		assert_eq!(c.get::<i32>(), Some(&123));

		assert!(a.get::<i32>().is_none());
		assert!(a.get::<bool>().is_none());
	}

	#[test]
	fn any_equal() {
		let a = Value::new("abc".to_string());
		let b = Value::new("abc".to_string());
		let c = Value::new("123".to_string());
		let d = Value::new(42);
		let e = Value::new(42);
		assert_eq!(a, b);
		assert_eq!(d, e);
		assert!(a != c);
		assert!(a != d);
	}

	#[test]
	fn any_display() {
		let a = Value::new("abc");
		assert_eq!(format!("{a}"), "abc");

		let a = Value::new(42);
		assert_eq!(format!("{a}"), "42");
	}
}
