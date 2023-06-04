use super::*;

/// Trait representing any integer type.
pub trait IntType: IsType {
	fn op_minus(v: Self::Data) -> Result<Self::Data>;

	fn op_add(a: Self::Data, b: Self::Data) -> Self::Data;
	fn op_sub(a: Self::Data, b: Self::Data) -> Self::Data;

	fn from_value(value: &Value) -> Result<Self::Data> {
		if let Some(value) = value.get::<Self::Data>() {
			Ok(*value)
		} else {
			todo!()
		}
	}
}

int_type!(Isize: isize, i);
int_type!(Usize: usize, u);

int_type!(I32: i32, i);
int_type!(U32: u32, u);

int_type!(I64: i64, i);
int_type!(U64: u64, u);

//====================================================================================================================//
// Operators
//====================================================================================================================//

/// Integer minus operator.
#[derive(Copy, Clone, Debug)]
pub struct IntMinus;

has_traits!(IntMinus);

impl<T: IntType> IsUnaryOp<T> for IntMinus {
	fn eval(&self, scope: &mut Scope, value: Value) -> Result<Value> {
		let _ = scope;
		let value = T::from_value(&value)?;
		let value = T::op_minus(value)?;
		Ok(Value::from(value))
	}
}

/// Integer addition operator.
#[derive(Copy, Clone, Debug)]
pub struct IntAdd;

has_traits!(IntAdd);

impl<T: IntType> IsBinaryOp<T> for IntAdd {
	fn eval(&self, scope: &mut Scope, lhs: Value, rhs: Value) -> Result<Value> {
		let _ = scope;
		let lhs = T::from_value(&lhs)?;
		let rhs = T::from_value(&rhs)?;
		Ok(Value::from(T::op_add(lhs, rhs)))
	}
}

/// Integer subtraction operator.
#[derive(Copy, Clone, Debug)]
pub struct IntSub;

has_traits!(IntSub);
impl<T: IntType> IsBinaryOp<T> for IntSub {
	fn eval(&self, scope: &mut Scope, lhs: Value, rhs: Value) -> Result<Value> {
		let _ = scope;
		let lhs = T::from_value(&lhs)?;
		let rhs = T::from_value(&rhs)?;
		Ok(Value::from(T::op_sub(lhs, rhs)))
	}
}

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

			impl IsType for $name {
				type Data = $int;

				fn new_value(&self, _scope: &mut Scope, data: &Self::Data) -> Result<Value> {
					Ok(Value::from(*data))
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
		($name:ident : $int:ty) => {
			impl IntType for $name {
				fn op_minus(_v: Self::Data) -> Result<Self::Data> {
					let typ = $name::default();
					Err(Errors::from(format!("minus operator for {typ} is invalid")))
				}

				fn op_add(a: Self::Data, b: Self::Data) -> Self::Data {
					a + b
				}

				fn op_sub(a: Self::Data, b: Self::Data) -> Self::Data {
					a - b
				}
			}
		};
	}

	#[macro_export]
	macro_rules! int_type_signed {
		($name:ident : $int:ty) => {
			impl IntType for $name {
				fn op_minus(v: Self::Data) -> Result<Self::Data> {
					Ok(-v)
				}

				fn op_add(a: Self::Data, b: Self::Data) -> Self::Data {
					a + b
				}

				fn op_sub(a: Self::Data, b: Self::Data) -> Self::Data {
					a - b
				}
			}
		};
	}

	pub use int_type;
}

use macros::*;
