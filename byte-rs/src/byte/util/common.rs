//! Common traits and implementation for standard types.

use std::sync::Arc;

use super::*;

//====================================================================================================================//
// Equality
//====================================================================================================================//

pub trait WithEquality {
	fn is_equal(&self, other: &Value) -> bool;
}

impl<T: IsValue + PartialEq> WithEquality for T {
	fn is_equal(&self, other: &Value) -> bool {
		if let Some(other) = other.get::<T>() {
			self == other
		} else {
			false
		}
	}
}

impl Value {
	pub fn with_equality(&self) -> Option<&dyn WithEquality> {
		get_trait!(self, WithEquality)
	}
}

impl PartialEq for Value {
	fn eq(&self, other: &Self) -> bool {
		if let Some(comparable) = self.with_equality() {
			comparable.is_equal(other)
		} else {
			Arc::as_ptr(&self.inner()) == Arc::as_ptr(&other.inner())
		}
	}
}

impl Eq for Value {}

//====================================================================================================================//
// Debug & Format
//====================================================================================================================//

pub trait WithDebug {
	fn fmt_debug(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result;
}

impl<T: IsValue + std::fmt::Debug> WithDebug for T {
	fn fmt_debug(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		self.fmt(f)
	}
}

pub trait WithDisplay {
	fn fmt_display(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result;
}

impl<T: IsValue + std::fmt::Display> WithDisplay for T {
	fn fmt_display(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		self.fmt(f)
	}
}

impl Value {
	pub fn with_debug(&self) -> Option<&dyn WithDebug> {
		get_trait!(self, WithDebug)
	}

	pub fn with_display(&self) -> Option<&dyn WithDisplay> {
		get_trait!(self, WithDisplay)
	}
}

impl std::fmt::Debug for Value {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		if let Some(value) = self.with_debug() {
			value.fmt_debug(f)
		} else {
			let ptr = Arc::as_ptr(self.inner());
			write!(f, "Value({}: {ptr:?})", self.type_name())
		}
	}
}

impl std::fmt::Display for Value {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		if let Some(value) = self.with_display() {
			value.fmt_display(f)
		} else {
			write!(f, "Value({})", self.type_name())
		}
	}
}

//====================================================================================================================//
// Traits for standard types
//====================================================================================================================//

has_traits!((): WithEquality, WithDebug);
has_traits!(String: WithEquality, WithDisplay, WithDebug);
has_traits!(bool: WithEquality, WithDisplay, WithDebug);
has_traits!(u8: WithEquality, WithDisplay, WithDebug);
has_traits!(i8: WithEquality, WithDisplay, WithDebug);
has_traits!(u16: WithEquality, WithDisplay, WithDebug);
has_traits!(i16: WithEquality, WithDisplay, WithDebug);
has_traits!(u32: WithEquality, WithDisplay, WithDebug);
has_traits!(i32: WithEquality, WithDisplay, WithDebug);
has_traits!(u64: WithEquality, WithDisplay, WithDebug);
has_traits!(i64: WithEquality, WithDisplay, WithDebug);
has_traits!(u128: WithEquality, WithDisplay, WithDebug);
has_traits!(i128: WithEquality, WithDisplay, WithDebug);
has_traits!(usize: WithEquality, WithDisplay, WithDebug);
has_traits!(isize: WithEquality, WithDisplay, WithDebug);
has_traits!(f32: WithEquality, WithDisplay, WithDebug);
has_traits!(f64: WithEquality, WithDisplay, WithDebug);
has_traits!(&'static str: WithEquality, WithDisplay, WithDebug);
