/// Execute a block of code if the input generic variable is of a given type.
///
/// ## Example
///
/// ```
/// fn some_func<T>(input: T) -> String {
///     when_type!(input: T =>
///         i32 {
///             let double = input * 2;
///             return format!("double of i32 = {double}");
///         }
///         i64 {
///             return format!("{input} is i64");
///         }
///     );
///
///     format!("invalid type")
/// }
/// ```
#[macro_export]
macro_rules! when_type {
	($id:ident : $from:ty => $($to:ty $blk:block)+) => {{
		let tf = ::std::any::TypeId::of::<$from>();
		$({
			let tt = ::std::any::TypeId::of::<$to>();
			if tf == tt {
				let $id: $to = unsafe { ::std::mem::transmute_copy(&($id)) };
				$blk
			}
		})*
	}};
}

pub use when_type;

#[cfg(test)]
mod tests {
	use super::*;

	fn check_when_type<T: 'static>(input: T) -> String {
		when_type!(input: T =>
			i32 {
				let double = input * 2;
				return format!("double of i32 = {double}");
			}

			i64 {
				return format!("{input} is i64");
			}
		);

		format!("invalid type")
	}

	#[test]
	fn when_type() {
		assert_eq!(check_when_type(10i32), "double of i32 = 20");
		assert_eq!(check_when_type(42i64), "42 is i64");
		assert_eq!(check_when_type(true), "invalid type");
	}
}
