use std::{
	any::{Any, TypeId},
	rc::Rc,
};

#[derive(Clone, Debug)]
pub struct Value {
	type_id: TypeId,
	value: Rc<dyn Any>,
}

#[allow(unused)]
impl Value {
	pub fn new<T: 'static>(value: T) -> Value {
		Value {
			type_id: TypeId::of::<Self>(),
			value: Rc::new(value),
		}
	}

	pub fn is<T: Valued>(&self) -> bool {
		self.type_id == TypeId::of::<T>()
	}

	pub fn get<T: Valued>(&self) -> Option<&T::Value> {
		if self.type_id == TypeId::of::<T>() {
			self.value.downcast_ref()
		} else {
			None
		}
	}

	pub fn value<T: 'static>(&self) -> Option<&T> {
		self.value.downcast_ref()
	}
}

impl PartialEq for Value {
	fn eq(&self, other: &Self) -> bool {
		self.type_id == other.type_id && {
			let a = self.value.as_ref();
			let b = other.value.as_ref();
			std::ptr::eq(a, b)
		}
	}
}

impl Eq for Value {}

pub trait Valued: Sized + 'static {
	type Value: Sized;

	fn new(value: Self::Value) -> Value {
		Value {
			type_id: TypeId::of::<Self>(),
			value: Rc::new(Box::new(value)),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn any_value() {
		let a = Value::new("abc".to_string());
		let b = Value::new("123".to_string());
		let c = Value::new(123);

		assert_eq!(a.value::<String>(), Some(&"abc".into()));
		assert_eq!(b.value::<String>(), Some(&"123".into()));
		assert_eq!(c.value::<i32>(), Some(&123));

		assert!(a.value::<i32>().is_none());
		assert!(a.value::<bool>().is_none());
	}
}
