use super::*;

use int::*;

pub struct OpOr {
	output: Type,
	eval_fn: fn(&mut RuntimeScope, &Expr) -> Result<bool>,
}

impl Debug for OpOr {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		write!(f, "OpOr")
	}
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
					eval_fn: IntegerToBoolean::eval_for(int_type),
				})
			} else {
				None
			};
		}

		match output {
			Type::Value(value) => match value {
				ValueType::Bool => Some(Self {
					output,
					eval_fn: BooleanEval::eval,
				}),
				ValueType::Int(int) => Some(Self {
					output,
					eval_fn: IntegerToBoolean::eval_for(&int),
				}),
				_ => None,
			},
			_ => None,
		}
	}
}

impl IsBinaryOp for OpOr {
	fn execute(&self, scope: &mut RuntimeScope, lhs: &Expr, rhs: &Expr) -> Result<ExprValue> {
		let lhs = (self.eval_fn)(scope, lhs)?;
		let result = if !lhs {
			let rhs = (self.eval_fn)(scope, rhs)?;
			rhs
		} else {
			true
		};
		Ok(Value::from(result).into())
	}

	fn get_type(&self) -> Type {
		self.output.clone()
	}
}

pub struct BooleanEval;

impl BooleanEval {
	pub fn eval(scope: &mut RuntimeScope, expr: &Expr) -> Result<bool> {
		let value = expr.execute(scope)?.value();
		value.to_bool()
	}
}

pub struct IntegerToBoolean;

impl IntegerToBoolean {
	pub fn eval_for(int: &IntType) -> fn(&mut RuntimeScope, &Expr) -> Result<bool> {
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

	fn eval<T: IsIntType>(scope: &mut RuntimeScope, expr: &Expr) -> Result<bool> {
		let value = expr.execute(scope)?.value();
		value
			.to_bool()
			.or_else(|_| T::from_value(&value).map(|x| !T::is_zero(x)))
	}
}
