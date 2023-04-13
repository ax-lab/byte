use std::any::TypeId;

use once_cell::sync::OnceCell;

use crate::core::str::*;

use super::*;

pub struct StrValue;

impl IsValue for Str {}

impl StrValue {
	pub fn get_type() -> Type {
		static TYPE: OnceCell<Type> = OnceCell::new();
		TYPE.get_or_init(|| Type::new(StrValue)).clone()
	}

	pub fn get(value: &Value) -> Option<&Str> {
		value.as_ref()
	}

	pub fn new(value: Str) -> Value {
		Value(Self::get_type(), InnerValue::pack(value))
	}

	fn inner(value: &InnerValue) -> &Str {
		unsafe { value.as_ref().unwrap() }
	}

	fn inner_mut(value: &mut InnerValue) -> &mut Str {
		unsafe { value.as_mut().unwrap() }
	}
}

impl IsType for StrValue {
	fn val_type_id(&self) -> std::any::TypeId {
		TypeId::of::<Str>()
	}

	fn fmt_val(&self, value: &InnerValue, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let value = Self::inner(value);
		write!(f, "{value}")
	}

	fn drop_val(&self, value: &mut InnerValue) {
		let value = std::mem::take(value);
		let value = unsafe { value.unpack::<Str>() };
		drop(value);
	}

	fn clone_val(&self, value: &InnerValue) -> InnerValue {
		let value = Self::inner(value);
		let value = value.clone();
		InnerValue::pack(value)
	}
}

impl std::fmt::Display for StrValue {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "String")
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn string_value() {
		let a = StrValue::new(Str::Static("abc"));
		let b = StrValue::new(Str::from("123".to_string()));
		assert_eq!(format!("{a}{b}"), "abc123");

		let xa = StrValue::get(&a).map(|x| x.as_str());
		let xb = StrValue::get(&b).map(|x| x.as_str());
		assert_eq!(xa, Some("abc"));
		assert_eq!(xb, Some("123"));

		let c = a.clone();
		assert_eq!(format!("{a}"), "abc");
		assert_eq!(format!("{c}"), "abc");
	}
}
