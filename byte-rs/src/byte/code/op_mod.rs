use super::*;

use int::*;

#[derive(Debug)]
pub struct OpMod {
	output: Type,
	eval_fn: fn(Value, Value) -> Result<Value>,
}

has_traits!(OpMod: IsBinaryOp);

impl OpMod {
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
				ValueType::Int(int) => Some(Self {
					output,
					eval_fn: IntegerMod::eval_for(&int),
				}),
				ValueType::Float(_) => todo!(),
				_ => None,
			},
			_ => None,
		}
	}
}

impl IsBinaryOp for OpMod {
	fn execute(&self, scope: &mut RuntimeScope, lhs: &Expr, rhs: &Expr) -> Result<ExprValue> {
		let lhs = lhs.execute(scope)?.into();
		let rhs = rhs.execute(scope)?.into();
		(self.eval_fn)(lhs, rhs).map(|x| x.into())
	}

	fn get_type(&self) -> Type {
		self.output.clone()
	}
}

struct IntegerMod;

impl IntegerMod {
	fn eval<T: IsIntType>(lhs: Value, rhs: Value) -> Result<Value> {
		let lhs = T::from_value(&lhs)?;
		let rhs = T::from_value(&rhs)?;
		let out = Value::from(T::op_mod(lhs, rhs));
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
