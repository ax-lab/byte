use super::*;

/// Trait representing any integer type.
pub trait IsIntType {
	type Data: Copy + Clone + IsValue;

	fn new_value(v: u128) -> Result<Self::Data>;

	fn op_add(a: Self::Data, b: Self::Data) -> Self::Data;
	fn op_sub(a: Self::Data, b: Self::Data) -> Self::Data;

	fn op_mul(a: Self::Data, b: Self::Data) -> Self::Data;
	fn op_div(a: Self::Data, b: Self::Data) -> Self::Data;

	fn from_value(value: &Value) -> Result<Self::Data> {
		if let Some(value) = value.get::<Self::Data>() {
			Ok(*value)
		} else {
			let typ = std::any::type_name::<Self::Data>();
			let error = format!("`{value:?}` is not a valid {typ}");
			let error = Errors::from(error);
			Err(error)
		}
	}
}

pub trait IsSignedIntType: IsIntType {
	fn op_minus(v: Self::Data) -> Result<Self::Data>;
}

int_type!(Isize: isize, i);
int_type!(Usize: usize, u);

int_type!(I8: i8, i);
int_type!(U8: u8, u);

int_type!(I16: i16, i);
int_type!(U16: u16, u);

int_type!(I32: i32, i);
int_type!(U32: u32, u);

int_type!(I64: i64, i);
int_type!(U64: u64, u);

int_type!(I128: i128, i);
int_type!(U128: u128, u);

//====================================================================================================================//
// Trait macro
//====================================================================================================================//

mod macros {
	#[macro_export]
	macro_rules! int_type {
		($name:ident : $int:ty, u) => {
			int_type_all!($name: $int);
			int_type_unsigned!($name: $int);
		};

		($name:ident : $int:ty, i) => {
			int_type_all!($name: $int);
			int_type_signed!($name: $int);
		};
	}

	#[macro_export]
	macro_rules! int_type_all {
		($name:ident : $int:ty) => {
			#[derive(Default, Copy, Clone, Eq, PartialEq, Debug)]
			pub struct $name;

			impl IsIntType for $name {
				type Data = $int;

				fn new_value(v: u128) -> Result<Self::Data> {
					if v <= Self::Data::MAX as u128 {
						Ok(v as Self::Data)
					} else {
						Err(Errors::from(format!("value overflows {}", stringify!($int))))
					}
				}

				fn op_add(a: Self::Data, b: Self::Data) -> Self::Data {
					a + b
				}

				fn op_sub(a: Self::Data, b: Self::Data) -> Self::Data {
					a - b
				}

				fn op_mul(a: Self::Data, b: Self::Data) -> Self::Data {
					a * b
				}

				fn op_div(a: Self::Data, b: Self::Data) -> Self::Data {
					a / b
				}
			}

			has_traits!($name);

			impl std::fmt::Display for $name {
				fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
					let name = stringify!($name);
					write!(f, "{name}")
				}
			}
		};
	}

	#[macro_export]
	macro_rules! int_type_unsigned {
		($name:ident : $int:ty) => {};
	}

	#[macro_export]
	macro_rules! int_type_signed {
		($name:ident : $int:ty) => {
			impl IsSignedIntType for $name {
				fn op_minus(v: Self::Data) -> Result<Self::Data> {
					Ok(-v)
				}
			}
		};
	}

	pub use int_type;
}

use macros::*;
