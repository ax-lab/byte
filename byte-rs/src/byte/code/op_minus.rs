use super::*;

use int::*;

#[derive(Debug)]
pub struct OpMinus {
	output: Type,
	eval_fn: fn(Value) -> Result<Value>,
}

has_traits!(OpMinus: IsUnaryOp);

impl OpMinus {
	pub fn for_type(arg: &Type) -> Option<Self> {
		let output = arg.clone();
		match output {
			Type::Value(value) => match value {
				ValueType::Int(int) => IntegerMinus::eval_for(&int).map(|eval_fn| Self { output, eval_fn }),
				ValueType::Float(_) => todo!(),
				_ => None,
			},
			_ => None,
		}
	}
}

impl IsUnaryOp for OpMinus {
	fn execute(&self, scope: &mut RuntimeScope, arg: &Expr) -> Result<Value> {
		let arg = arg.execute(scope)?;
		(self.eval_fn)(arg)
	}

	fn get_type(&self) -> Type {
		self.output.clone()
	}
}

struct IntegerMinus;

impl IntegerMinus {
	fn eval<T: IsSignedIntType>(arg: Value) -> Result<Value> {
		let arg = T::from_value(&arg)?;
		let out = Value::from(T::op_minus(arg));
		Ok(out)
	}

	fn eval_for(int: &IntType) -> Option<fn(Value) -> Result<Value>> {
		let result = match int {
			IntType::I8 => Self::eval::<I8>,
			IntType::I16 => Self::eval::<I16>,
			IntType::I32 => Self::eval::<I32>,
			IntType::I64 => Self::eval::<I64>,
			IntType::I128 => Self::eval::<I128>,
			IntType::U8 => return None,
			IntType::U16 => return None,
			IntType::U32 => return None,
			IntType::U64 => return None,
			IntType::U128 => return None,
		};
		Some(result)
	}
}
