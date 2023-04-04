use std::{
	any::TypeId,
	fmt::{Debug, Display},
	rc::Rc,
};

/// Provides ref-counted storage for a dynamically typed value.
#[derive(Clone, Debug)]
pub struct Value {
	type_id: TypeId,
	value: Rc<dyn IsValue>,
}

#[allow(unused)]
impl Value {
	pub fn new<T: IsValue>(value: T) -> Value {
		Value {
			type_id: TypeId::of::<T>(),
			value: Rc::new(value),
		}
	}

	pub fn is<T: IsValue>(&self) -> bool {
		self.type_id == TypeId::of::<T>()
	}

	pub fn get<T: 'static>(&self) -> Option<&T> {
		if self.type_id == TypeId::of::<T>() {
			let ptr = self.value.as_ref();
			let ptr = unsafe { &*(ptr as *const dyn IsValue as *const T) };
			Some(ptr)
		} else {
			None
		}
	}
}

/// This trait provides the minimum features required for a [`Value`]. It is
/// automatically implemented for `Display + Debug + Eq` types.
pub trait IsValue: Debug + 'static {
	fn output(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result;

	fn is_eq(&self, other: &Value) -> bool;
}

impl<T: Display + Debug + Eq + 'static> IsValue for T {
	fn output(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{self}")
	}

	fn is_eq(&self, other: &Value) -> bool {
		if let Some(other) = other.get::<Self>() {
			self == other
		} else {
			false
		}
	}
}

impl Display for Value {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.value.output(f)
	}
}

impl PartialEq for Value {
	fn eq(&self, other: &Self) -> bool {
		self.value.is_eq(other)
	}
}

impl Eq for Value {}

#[cfg(test)]
mod tests {
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
