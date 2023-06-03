use std::{
	any::{Any, TypeId},
	sync::Arc,
};

use super::*;

/// Trait for types compatible with dynamic typing and multi-threading.
///
/// This is similar to [`IsValue`] but without the requirement for traits.
pub trait Cell: Any + Send + Sync {}

impl<T: Any + Send + Sync> Cell for T {}

/// Trait for any value that can be dynamically typed using a [`Value`].
pub trait IsValue: Any + Send + Sync + HasTraits {
	fn as_value(&self) -> &dyn IsValue;
}

impl<T: Any + Send + Sync + HasTraits> IsValue for T {
	fn as_value(&self) -> &dyn IsValue {
		self
	}
}

//====================================================================================================================//
// Value type
//====================================================================================================================//

/// An immutable container for any [`IsValue`] supporting dynamic typing and
/// traits.
#[derive(Clone)]
pub struct Value {
	inner: Arc<dyn IsValue>,
}

impl Value {
	pub fn from<T: IsValue>(value: T) -> Self {
		if TypeId::of::<T>() == TypeId::of::<Value>() {
			let ptr = &value;
			let ptr: &Value = unsafe { std::mem::transmute(ptr) };
			return ptr.clone();
		}

		Self {
			inner: Arc::new(value),
		}
	}

	pub fn get<T: IsValue>(&self) -> Option<&T> {
		let data = self.inner.as_ref();
		if data.type_id() == TypeId::of::<T>() {
			let data = unsafe {
				let data = data as *const dyn IsValue as *const T;
				data.as_ref().unwrap()
			};
			Some(data)
		} else {
			None
		}
	}

	pub fn type_name(&self) -> &'static str {
		self.inner.type_name()
	}

	pub fn cloned<T: IsValue + Clone>(&self) -> Option<T> {
		self.get().cloned()
	}

	pub(crate) fn inner(&self) -> &Arc<dyn IsValue> {
		&self.inner
	}
}

impl HasTraits for Value {
	fn get_trait(&self, type_id: std::any::TypeId) -> Option<&dyn HasTraits> {
		self.inner.get_trait(type_id)
	}
}

//====================================================================================================================//
// Tests
//====================================================================================================================//

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_simple() {
		let a = Value::from(true);
		let b = Value::from(false);
		let c = Value::from(42);
		let d = Value::from("abc");
		let e = Value::from("123".to_string());

		// Basic comparisons

		assert_eq!(a.cloned(), Some(true));
		assert_eq!(b.cloned(), Some(false));
		assert_eq!(c.cloned(), Some(42));
		assert_eq!(d.cloned(), Some("abc"));
		assert_eq!(e.get(), Some(&String::from("123")));

		assert!(a.get::<i32>().is_none());

		assert_eq!(a, Value::from(true));
		assert_ne!(a, Value::from(false));

		// Pointer comparison

		struct NonComparable;

		has_traits!(NonComparable);

		let a = Value::from(NonComparable);
		let b = Value::from(NonComparable);
		let c = a.clone();
		assert_eq!(a, a);
		assert_eq!(a, c);
		assert_ne!(a, b);
	}

	#[test]
	fn test_format() {
		let a = Value::from(true);
		let b = Value::from(false);
		let c = Value::from(42);
		let d = Value::from("abc");
		let e = Value::from("123".to_string());

		assert_eq!(format!("{a}"), "true");
		assert_eq!(format!("{b}"), "false");
		assert_eq!(format!("{c}"), "42");
		assert_eq!(format!("{d}"), "abc");
		assert_eq!(format!("{e}"), "123");

		assert_eq!(format!("{d:?}"), "\"abc\"");
		assert_eq!(format!("{e:?}"), "\"123\"");

		struct NoDisplay;

		has_traits!(NoDisplay);

		let x = Value::from(NoDisplay);

		let fx = format!("{x}");
		assert!(
			fx.contains("Value(") && fx.contains("NoDisplay") && fx.contains(")"),
			"invalid format: {fx}"
		);

		let fx = format!("{x:?}");
		assert!(
			fx.contains("Value(")
				&& fx.contains("NoDisplay")
				&& fx.contains(": 0x")
				&& fx.contains(")"),
			"invalid debug format: {fx}"
		);
	}
}
