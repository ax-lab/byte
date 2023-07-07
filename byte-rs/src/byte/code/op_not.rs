use super::*;

#[derive(Debug)]
pub struct OpNot {
	output: Type,
	eval_fn: fn(Value) -> Result<Value>,
}

impl OpNot {
	pub fn for_type(arg: &Type) -> Option<Self> {
		let output = arg.clone();
		match output {
			Type::Value(value) => match value {
				ValueType::Bool => Some(Self {
					output,
					eval_fn: BooleanNot::eval,
				}),
				_ => None,
			},
			_ => None,
		}
	}
}

impl IsUnaryOp for OpNot {
	fn execute(&self, scope: &mut RuntimeScope, arg: &Expr) -> Result<ExprValue> {
		let arg = arg.execute(scope)?.into();
		(self.eval_fn)(arg).map(|x| x.into())
	}

	fn get_type(&self) -> Type {
		self.output.clone()
	}
}

struct BooleanNot;

impl BooleanNot {
	fn eval(arg: Value) -> Result<Value> {
		let arg = arg.to_bool()?;
		Ok(Value::from(!arg))
	}
}
