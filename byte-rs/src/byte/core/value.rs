use std::{
	any::{Any, TypeId},
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

	fn span(&self) -> Option<Span> {
		if let Some(value) = get_trait!(self, WithSpan) {
			value.get_span()
		} else {
			None
		}
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
	data: Arc<dyn IsValue>,
}

impl Value {
	pub fn from<T: IsValue>(value: T) -> Self {
		Self {
			data: Arc::new(value),
		}
	}

	pub fn get<T: IsValue>(&self) -> Option<&T> {
		let data = self.data.as_ref();
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
}

fmt_from_repr!(Value);

impl HasTraits for Value {
	fn get_trait(&self, type_id: std::any::TypeId) -> Option<&dyn HasTraits> {
		self.data.get_trait(type_id)
	}
}

impl HasRepr for Value {
	fn output_repr(&self, output: &mut Repr) -> std::io::Result<()> {
		self.data.output_repr(output)
	}
}

impl PartialEq for Value {
	fn eq(&self, other: &Self) -> bool {
		if let Some(comparable) = self.with_equality() {
			comparable.is_equal(other)
		} else {
			Arc::as_ptr(&self.data) == Arc::as_ptr(&other.data)
		}
	}
}

impl Eq for Value {}

//====================================================================================================================//
// Var type
//====================================================================================================================//

pub struct Var {}

impl Var {
	pub fn value(&self) -> Value {
		todo!()
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
}
