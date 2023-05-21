use std::{
	any::{Any, TypeId},
	collections::HashMap,
	sync::Arc,
};

use super::*;

//====================================================================================================================//
// IsValue trait
//====================================================================================================================//

pub trait IsValue: Any + Send + Sync + HasRepr + HasTraits {
	fn as_value(&self) -> &dyn IsValue;

	fn with_equality(&self) -> Option<&dyn WithEquality> {
		get_trait!(self, WithEquality)
	}
}

impl<T: Any + Send + Sync + HasRepr + HasTraits> IsValue for T {
	fn as_value(&self) -> &dyn IsValue {
		self
	}
}

//====================================================================================================================//
// Value type
//====================================================================================================================//

/// An immutable container for any [`IsValue`] with support for dynamic typing.
#[derive(Clone)]
pub struct Value {
	inner: Arc<InnerValue>,
}

#[derive(Clone)]
struct InnerValue {
	value: Arc<dyn IsValue>,
	table: HashMap<TypeId, Value>,
}

impl Value {
	pub fn from<T: IsValue>(value: T) -> Self {
		if TypeId::of::<T>() == TypeId::of::<Value>() {
			let ptr = &value;
			let ptr: &Value = unsafe { std::mem::transmute(ptr) };
			return ptr.clone();
		}

		Self {
			inner: Arc::new(InnerValue {
				value: Arc::new(value),
				table: Default::default(),
			}),
		}
	}

	pub fn get<T: IsValue>(&self) -> Option<&T> {
		let data = self.inner.value.as_ref();
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

	pub fn cloned<T: IsValue + Clone>(&self) -> Option<T> {
		self.get().cloned()
	}

	pub fn with_field<T: IsValue>(&self, data: T) -> Value {
		let key = TypeId::of::<T>();
		let val = Value::from(data);
		let mut value = self.clone();
		let inner = Arc::make_mut(&mut value.inner);
		inner.table.insert(key, val);
		value
	}

	pub fn has_field<T: IsValue>(&self) -> bool {
		self.get_field::<T>().is_some()
	}

	pub fn get_field<T: IsValue>(&self) -> Option<&T> {
		let key = TypeId::of::<T>();
		self.inner.table.get(&key).and_then(|v| v.get::<T>())
	}
}

fmt_from_repr!(Value);

impl HasTraits for Value {
	fn get_trait(&self, type_id: std::any::TypeId) -> Option<&dyn HasTraits> {
		self.inner.value.get_trait(type_id)
	}
}

impl HasRepr for Value {
	fn output_repr(&self, output: &mut Repr) -> std::io::Result<()> {
		self.inner.value.output_repr(output)
	}
}

impl PartialEq for Value {
	fn eq(&self, other: &Self) -> bool {
		if let Some(comparable) = self.with_equality() {
			comparable.is_equal(other)
		} else {
			Arc::as_ptr(&self.inner) == Arc::as_ptr(&other.inner)
		}
	}
}

impl Eq for Value {}

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
		assert_eq!(a.cloned(), Some(true));
		assert_eq!(b.cloned(), Some(false));
		assert_eq!(c.cloned(), Some(42));
		assert_eq!(d.cloned(), Some("abc"));
		assert_eq!(e.get(), Some(&String::from("123")));

		assert!(a.get::<i32>().is_none());

		assert_eq!(a, Value::from(true));
		assert_ne!(a, Value::from(false));

		struct NonComparable;

		has_traits!(NonComparable);

		impl HasRepr for NonComparable {
			fn output_repr(&self, output: &mut Repr<'_>) -> std::io::Result<()> {
				use std::io::Write;
				write!(output, "NonComparable")
			}
		}

		let a = Value::from(NonComparable);
		let b = Value::from(NonComparable);
		let c = a.clone();
		assert_eq!(a, a);
		assert_eq!(a, c);
		assert_ne!(a, b);
	}

	#[test]
	fn test_associated() {
		let a = Value::from(1);
		let b = Value::from(2);
		let c = a.clone();

		let a = a.with_field("abc");

		assert_eq!(a.get_field::<&str>().cloned(), Some("abc"));
		assert_eq!(b.get_field::<&str>(), None);
		assert_eq!(c.get_field::<&str>(), None);

		let x = a.with_field("123");
		assert_eq!(a.get_field::<&str>().cloned(), Some("abc"));
		assert_eq!(x.get_field::<&str>().cloned(), Some("123"));
	}
}
