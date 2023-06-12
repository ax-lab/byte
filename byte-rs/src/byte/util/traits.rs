use super::*;

/// Provides dynamic typing for traits.
///
/// To implement this trait use the [`has_traits`] or [`with_trait`] macros.
///
/// To retrieve a trait from a [`HasTraits`] reference use [`get_trait`].
pub trait HasTraits {
	fn type_name(&self) -> &'static str {
		std::any::type_name::<Self>()
	}

	fn get_trait(&self, type_id: TypeId) -> Option<&dyn HasTraits> {
		let _ = type_id;
		None
	}
}

impl<T: HasTraits> HasTraits for Option<T> {
	fn get_trait(&self, type_id: TypeId) -> Option<&dyn HasTraits> {
		if let Some(value) = self {
			value.get_trait(type_id)
		} else {
			None
		}
	}
}

//====================================================================================================================//
// Helper macros
//====================================================================================================================//

/// Implements the [`HasTraits`] macros for a type, expanding to an `impl`
/// block.
///
/// # Examples
///
/// ```
/// # use byte::traits::*;
/// #
/// struct SomeType {}
///
/// trait A {
///     fn a(&self) -> u32;
/// }
///
/// trait B {
///     fn b(&self) {}
/// }
///
/// has_traits!(SomeType: A, B);
///
/// impl A for SomeType {
///     fn a(&self) -> u32 { 42 }
/// }
///
/// impl B for SomeType {}
///
/// fn somewhere(value: &dyn HasTraits) {
///     let value_a = get_trait!(value, A).unwrap();
///     assert_eq!(value_a.a(), 42);
///
///     let value_b = get_trait!(value, B);
///     assert!(value_b.is_none());
/// }
/// ```
#[macro_export]
macro_rules! has_traits {
	($me:ty $( : $($typ:path),+ )?) => {
		impl $crate::traits::HasTraits for $me {
			fn get_trait(
				&self,
				type_id: ::std::any::TypeId,
			) -> Option<&dyn $crate::traits::HasTraits> {
				let _ = type_id;
				$($(
					$crate::traits::with_trait!(self, type_id, $typ);
				)+)?
				None
			}
		}
	};

	(ref $me:ty $( : $($typ:path),+ )?) => {
		impl<'a> $crate::traits::HasTraits for $me {
			fn get_trait(
				&self,
				type_id: ::std::any::TypeId,
			) -> Option<&dyn $crate::traits::HasTraits> {
				let _ = type_id;
				$($(
					$crate::traits::with_trait!(self, type_id, $typ);
				)+)?
				None
			}
		}
	};
}

/// Retrieve a specific trait from a [`HasTraits`] type.
///
/// See [`has_traits`].
#[macro_export]
macro_rules! get_trait {
	($me:expr, $target:path) => {{
		use ::std::any::TypeId;
		use $crate::traits::HasTraits;

		fn cast_value(p: &dyn HasTraits) -> &dyn $target {
			unsafe { std::mem::transmute(p) }
		}

		let me = $me;
		let id = TypeId::of::<dyn $target>();
		HasTraits::get_trait(me, id).map(|x| cast_value(x))
	}};
}

/// Helper to implement a single type branch for a [`HasTraits`] type.
///
/// See [`has_traits`].
///
/// # Examples
///
/// ```
/// # use byte::traits::*;
/// # use ::std::any::TypeId;
/// #
/// struct SomeType {}
///
/// trait A {
///     fn a(&self) -> u32;
/// }
///
/// impl A for SomeType {
///     fn a(&self) -> u32 { 42 }
/// }
///
/// impl HasTraits for SomeType {
///     fn get_trait(&self, type_id: TypeId) -> Option<&dyn HasTraits> {
///         with_trait!(self, type_id, A);
///         None
///     }
/// }
///
/// fn use_a(value: &dyn HasTraits) {
///     let value_a = get_trait!(value, A).unwrap();
///     assert_eq!(value_a.a(), 42);
/// }
/// ```
#[macro_export]
macro_rules! with_trait {
	($me:expr, $type_id:ident, $typ:path) => {
		if ($type_id == ::std::any::TypeId::of::<dyn $typ>()) {
			unsafe {
				let me = $me as &dyn $typ;
				let me = std::mem::transmute(me);
				return Some(me);
			}
		}
	};
}

use std::any::TypeId;

pub use get_trait;
pub use has_traits;
pub use with_trait;

//====================================================================================================================//
// Tests
//====================================================================================================================//

#[cfg(test)]
mod tests {

	use super::*;

	//----------------------------------------------------------------------------------------------------------------//
	// Trait setup
	//----------------------------------------------------------------------------------------------------------------//

	trait A {
		fn a(&self) -> String;
	}

	trait B {
		fn b(&self) -> String;
	}

	trait C {
		fn c(&self) -> String;
	}

	trait X: HasTraits {
		fn as_a(&self) -> Option<&dyn A> {
			get_trait!(self, A)
		}

		fn as_b(&self) -> Option<&dyn B> {
			get_trait!(self, B)
		}

		fn as_c(&self) -> Option<&dyn C> {
			get_trait!(self, C)
		}
	}

	impl<T: HasTraits> X for T {}

	//----------------------------------------------------------------------------------------------------------------//
	// Concrete types
	//----------------------------------------------------------------------------------------------------------------//

	has_traits!(HasA: A);
	has_traits!(HasB: B);
	has_traits!(HasC: C);
	has_traits!(HasAB: A, B);

	impl HasTraits for HasABC {
		fn get_trait(&self, type_id: TypeId) -> Option<&dyn HasTraits> {
			with_trait!(self, type_id, A);
			with_trait!(self, type_id, B);
			with_trait!(self, type_id, C);
			None
		}
	}

	struct HasA(&'static str);

	impl A for HasA {
		fn a(&self) -> String {
			format!("A from {}", self.0)
		}
	}

	struct HasB(&'static str);

	impl B for HasB {
		fn b(&self) -> String {
			format!("B from {}", self.0)
		}
	}

	struct HasC(&'static str);

	impl C for HasC {
		fn c(&self) -> String {
			format!("C from {}", self.0)
		}
	}

	struct HasAB(&'static str);

	impl A for HasAB {
		fn a(&self) -> String {
			format!("A from {}", self.0)
		}
	}

	impl B for HasAB {
		fn b(&self) -> String {
			format!("B from {}", self.0)
		}
	}

	struct HasABC(&'static str);

	impl A for HasABC {
		fn a(&self) -> String {
			format!("A from {}", self.0)
		}
	}

	impl B for HasABC {
		fn b(&self) -> String {
			format!("B from {}", self.0)
		}
	}

	impl C for HasABC {
		fn c(&self) -> String {
			format!("C from {}", self.0)
		}
	}

	//----------------------------------------------------------------------------------------------------------------//
	// Test
	//----------------------------------------------------------------------------------------------------------------//

	#[test]
	fn test_has_traits() {
		let a: Box<dyn X> = Box::new(HasA("a"));
		let b: Box<dyn X> = Box::new(HasB("b"));
		let c: Box<dyn X> = Box::new(HasC("c"));
		let ab: Box<dyn X> = Box::new(HasAB("ab"));
		let abc: Box<dyn X> = Box::new(HasABC("abc"));

		assert_eq!(a.as_a().unwrap().a(), "A from a");
		assert_eq!(b.as_b().unwrap().b(), "B from b");
		assert_eq!(c.as_c().unwrap().c(), "C from c");

		assert_eq!(ab.as_a().unwrap().a(), "A from ab");
		assert_eq!(ab.as_b().unwrap().b(), "B from ab");

		assert_eq!(abc.as_a().unwrap().a(), "A from abc");
		assert_eq!(abc.as_b().unwrap().b(), "B from abc");
		assert_eq!(abc.as_c().unwrap().c(), "C from abc");

		assert!(a.as_b().is_none());
		assert!(a.as_c().is_none());

		assert!(b.as_a().is_none());
		assert!(b.as_c().is_none());

		assert!(c.as_a().is_none());
		assert!(c.as_b().is_none());

		assert!(ab.as_c().is_none());
	}
}
