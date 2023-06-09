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
// WithSpan
//====================================================================================================================//

pub trait WithSpan {
	fn span(&self) -> Option<&Span>;
}

impl Value {
	pub fn span(&self) -> Option<&Span> {
		get_trait!(self, WithSpan).and_then(|x| x.span())
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
