use super::*;

pub struct OpAnd {
	output: Type,
	eval_fn: fn(&mut RuntimeScope, &Expr) -> Result<bool>,
}

impl Debug for OpAnd {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		write!(f, "OpAnd")
	}
}

has_traits!(OpAnd: IsBinaryOp);

impl OpAnd {
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

impl IsBinaryOp for OpAnd {
	fn execute(&self, scope: &mut RuntimeScope, lhs: &Expr, rhs: &Expr) -> Result<ExprValue> {
		let lhs = (self.eval_fn)(scope, lhs)?;
		let result = if lhs {
			let rhs = (self.eval_fn)(scope, rhs)?;
			rhs
		} else {
			false
		};
		Ok(Value::from(result).into())
	}

	fn get_type(&self) -> Type {
		self.output.clone()
	}
}
