use super::*;

use int::*;

#[derive(Debug)]
pub struct OpAdd {
	output: Type,
	eval_fn: fn(Value, Value) -> Result<Value>,
}

has_traits!(OpAdd: IsBinaryOp);

impl OpAdd {
	pub fn for_type(lhs: &Type) -> Option<Self> {
		Self::for_types(lhs, lhs)
	}

	pub fn for_types(lhs: &Type, rhs: &Type) -> Option<Self> {
		if lhs != rhs {
			return None;
		}

		let output = lhs.clone();
		match output {
			Type::Value(value) => match value {
				ValueType::Unit => None,
				ValueType::Never => None,
				ValueType::Bool => None,
				ValueType::Str => todo!(),
				ValueType::Int(int) => Some(Self {
					output,
					eval_fn: Integer::eval_for(&int),
				}),
				ValueType::Float(_) => todo!(),
			},
		}
	}
}

impl IsBinaryOp for OpAdd {
	fn execute(&self, lhs: Value, rhs: Value) -> Result<Value> {
		(self.eval_fn)(lhs, rhs)
	}

	fn get_type(&self) -> Type {
		self.output.clone()
	}
}

struct Integer;

impl Integer {
	fn eval<T: IsIntType>(lhs: Value, rhs: Value) -> Result<Value> {
		let lhs = T::from_value(&lhs)?;
		let rhs = T::from_value(&rhs)?;
		let out = Value::from(T::op_add(lhs, rhs));
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