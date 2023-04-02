use std::any::{Any, TypeId};

use crate::core::context::*;

impl Context {
	pub fn new_any<T: 'static>(&self, value: T) -> AnyValue {
		let value = self.save(value);
		let value: &dyn Any = value;
		AnyValue { value }
	}
}

#[derive(Copy, Clone)]
pub struct AnyValue {
	value: &'static dyn Any,
}

impl AnyValue {
	pub fn get<T>(&self) -> Option<&'static T> {
		self.value.downcast_ref()
	}
}

#[derive(Copy, Clone)]
pub struct Value {
	type_id: TypeId,
	value: AnyValue,
}

impl Value {
	pub fn is<T: Valued>(&self) -> bool {
		self.type_id == TypeId::of::<T>()
	}

	pub fn get<T: Valued>(&self) -> Option<&'static T::Value> {
		if self.type_id == TypeId::of::<T>() {
			self.value.get::<T::Value>()
		} else {
			None
		}
	}

	pub fn val<T: Valued>(&self) -> Option<T::Value>
	where
		<T as Valued>::Value: Copy,
	{
		self.get::<T>().map(|x| x.clone())
	}
}

pub trait Valued: 'static {
	type Value: 'static;

	fn new(ctx: Context, value: Self::Value) -> Value {
		Value {
			type_id: TypeId::of::<Self>(),
			value: ctx.new_any(value),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn any_value() {
		let ctx = Context::new();
		let a = ctx.new_any("abc".to_string());
		let b = ctx.new_any("123".to_string());
		let c = ctx.new_any(123);

		assert_eq!(a.get::<String>(), Some(&"abc".into()));
		assert_eq!(b.get::<String>(), Some(&"123".into()));
		assert_eq!(c.get::<i32>(), Some(&123));

		assert!(a.get::<i32>().is_none());
		assert!(a.get::<bool>().is_none());
	}
}
