use super::*;

use int::*;

#[derive(Debug)]
pub struct OpNeg {
	output: Type,
	eval_fn: fn(Value) -> Result<Value>,
}

impl OpNeg {
	pub fn for_type(arg: &Type) -> Option<Self> {
		let output = arg.clone();
		match output {
			Type::Value(value) => match value {
				ValueType::Bool => Some(Self {
					output,
					eval_fn: BooleanNeg::eval,
				}),
				ValueType::Int(int) => Some(Self {
					output,
					eval_fn: IntegerNeg::eval_for(&int),
				}),
				ValueType::Float(_) => todo!(),
				_ => None,
			},
			_ => None,
		}
	}
}

impl IsUnaryOp for OpNeg {
	fn execute(&self, scope: &mut RuntimeScope, arg: &Expr) -> Result<ExprValue> {
		let arg = arg.execute(scope)?.into();
		(self.eval_fn)(arg).map(|x| x.into())
	}

	fn get_type(&self) -> Type {
		self.output.clone()
	}
}

struct BooleanNeg;

impl BooleanNeg {
	fn eval(arg: Value) -> Result<Value> {
		let arg = Self::to_bool(arg)?;
		Ok(Value::from(!arg))
	}

	fn to_bool(value: Value) -> Result<bool> {
		if let Some(value) = value.get::<bool>() {
			Ok(*value)
		} else {
			let error = format!("value `{value:?}` is neg a valid boolean");
			Err(Errors::from(error, Span::default()))
		}
	}
}

struct IntegerNeg;

impl IntegerNeg {
	fn eval<T: IsIntType>(arg: Value) -> Result<Value> {
		let arg = T::from_value(&arg)?;
		let out = Value::from(T::op_neg(arg));
		Ok(out)
	}

	fn eval_for(int: &IntType) -> fn(Value) -> Result<Value> {
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
