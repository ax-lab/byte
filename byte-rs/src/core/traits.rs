use std::any::TypeId;

use super::*;

/// Provides dynamic typing with traits.
///
/// To implement this trait use the [`has_traits`] macro. To retrieve a trait
/// from a [`HasTraits`] value use the [`to_trait`] macro.
pub trait HasTraits {
	fn type_name(&self) -> &'static str {
		std::any::type_name::<Self>()
	}

	fn get_trait(&self, type_id: TypeId) -> Option<&dyn HasTraits> {
		let _ = type_id;
		None
	}

	fn get_trait_mut(&mut self, type_id: TypeId) -> Option<&mut dyn HasTraits> {
		unsafe { std::mem::transmute(self.get_trait(type_id)) }
	}
}

/// Implements the [`HasTraits`] macros for a type, expanding to an `impl`
/// block.
///
/// # Examples
///
/// ```
/// struct SomeType {}
///
/// trait A {
///     fn a(&self) {}
/// }
///
/// trait B {
///     fn b(&self) {}
/// }
///
/// has_traits!(SomeType, A, B);
///
/// impl A for SomeType {}
/// impl B for SomeType {}
///
/// fn somewhere(value: &dyn HasTraits) {
///     let value = to_trait!(value, A).unwrap();
///     value.a();
/// }
/// ```
#[macro_export]
macro_rules! has_traits {
	($me:path $( : $($typ:path),+ )?) => {
		impl crate::core::traits::HasTraits for $me {
			fn get_trait(
				&self,
				type_id: ::std::any::TypeId,
			) -> Option<&dyn crate::core::traits::HasTraits> {
				let _ = type_id;
				$($(
					if (type_id == ::std::any::TypeId::of::<dyn $typ>()) {
						unsafe {
							let me = self as &dyn $typ;
							let me = std::mem::transmute(me);
							return Some(me);
						}
					}
				)+)?
				None
			}
		}
	};
}

#[macro_export]
macro_rules! get_trait {
	($me:expr, $target:path) => {{
		use crate::core::traits::HasTraits;
		use ::std::any::TypeId;

		fn cast_value(p: &dyn HasTraits) -> &dyn $target {
			unsafe { std::mem::transmute(p) }
		}

		let me = $me;
		let id = TypeId::of::<dyn $target>();
		HasTraits::get_trait(me, id).map(|x| cast_value(x))
	}};
}

#[macro_export]
macro_rules! get_trait_mut {
	($me:expr, $target:path) => {{
		use crate::core::traits::HasTraits;
		use ::std::any::TypeId;

		fn cast_value(p: &mut dyn HasTraits) -> &mut dyn $target {
			unsafe { std::mem::transmute(p) }
		}

		let me = $me;
		let id = TypeId::of::<dyn $target>();
		HasTraits::get_trait_mut(me, id).map(|x| cast_value(x))
	}};
}

#[macro_export]
macro_rules! some_trait {
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

pub use get_trait;
pub use get_trait_mut;
pub use has_traits;
pub use some_trait;

//--------------------------------------------------------------------------------------------------------------------//
// Tests
//--------------------------------------------------------------------------------------------------------------------//

#[cfg(test)]
mod tests {
	use std::sync::Arc;

	use super::*;

	#[test]
	fn test_has_traits() {
		let v = Arc::new(SomeType {
			a: "from A".into(),
			b: "from B".into(),
		});
		assert_eq!(v.say(), "from some type");

		let va = get_trait!(v.as_ref(), A).unwrap();
		let vb = get_trait!(v.as_ref(), B).unwrap();
		assert_eq!(va.a(), "from A");
		assert_eq!(vb.b(), "from B");

		let v = v.as_ref();
		assert!(get_trait!(v, C).is_none());
	}

	has_traits!(SomeType: A, B);

	trait A {
		fn a(&self) -> String;
	}

	trait B {
		fn b(&self) -> String;
	}

	trait C {
		fn c(&self) -> String;
	}

	#[derive(Debug, Clone, PartialEq)]
	struct SomeType {
		a: String,
		b: String,
	}

	impl SomeType {
		fn say(&self) -> String {
			format!("from some type")
		}
	}

	impl A for SomeType {
		fn a(&self) -> String {
			self.a.clone()
		}
	}

	impl B for SomeType {
		fn b(&self) -> String {
			self.b.clone()
		}
	}
}
