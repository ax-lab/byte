use super::*;

use int::*;

#[derive(Debug)]
pub struct OpOr {
	output: Type,
	eval_fn: fn(Value, Value) -> Result<Value>,
}

has_traits!(OpOr: IsBinaryOp);

impl OpOr {
	pub fn for_type(lhs: &Type) -> Option<Self> {
		Self::for_types(lhs, lhs)
	}

	pub fn for_types(lhs: &Type, rhs: &Type) -> Option<Self> {
		let output = Type::Value(ValueType::Bool);
		if lhs != rhs {
			return if (lhs.is_int() || lhs.is_boolean()) && (rhs.is_int() || rhs.is_boolean()) {
				let int_type = lhs.get_int_type().or_else(|| rhs.get_int_type()).unwrap();
				Some(Self {
					output,
					eval_fn: IntegerOr::eval_for(int_type),
				})
			} else {
				None
			};
		}

		match output {
			Type::Value(value) => match value {
				ValueType::Bool => Some(Self {
					output,
					eval_fn: BooleanOr::eval,
				}),
				ValueType::Int(int) => Some(Self {
					output,
					eval_fn: IntegerOr::eval_for(&int),
				}),
				_ => None,
			},
			_ => None,
		}
	}
}

impl IsBinaryOp for OpOr {
	fn execute(&self, lhs: Value, rhs: Value) -> Result<Value> {
		(self.eval_fn)(lhs, rhs)
	}

	fn get_type(&self) -> Type {
		self.output.clone()
	}
}

struct BooleanOr;

impl BooleanOr {
	fn eval(lhs: Value, rhs: Value) -> Result<Value> {
		let lhs = lhs.to_bool()?;
		let rhs = rhs.to_bool()?;
		let out = Value::from(lhs || rhs);
		Ok(out)
	}
}

struct IntegerOr;

impl IntegerOr {
	fn eval<T: IsIntType>(lhs: Value, rhs: Value) -> Result<Value> {
		let lhs = lhs.to_bool().or_else(|_| T::from_value(&lhs).map(|x| !T::is_zero(x)))?;
		let rhs = rhs.to_bool().or_else(|_| T::from_value(&rhs).map(|x| !T::is_zero(x)))?;
		let out = Value::from(lhs || rhs);
		Ok(out)
	}

	fn eval_for(int: &IntType) -> fn(Value, Value) -> Result<Value> {
		match int {
			IntType::I8 => Self::eval::<I8>,
			IntType::U8 => Self::eval::<U8>,
			IntType::I16 => Self::eval::<I16>,
			IntType::U16 => Self::eval::<U16>,
			IntType::I32 => Self::eval::<I32>,
			IntType::U32 => Self::eval::<U32>,
			IntType::I64 => Self::eval::<I64>,
			IntType::U64 => Self::eval::<U64>,
			IntType::I128 => Self::eval::<I128>,
			IntType::U128 => Self::eval::<U128>,
		}
	}
}
