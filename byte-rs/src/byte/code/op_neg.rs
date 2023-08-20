use super::*;

#[derive(Debug)]
pub struct OpNeg {
	output: Type,
}

impl OpNeg {
	pub fn for_type(arg: &Type) -> Option<Self> {
		let output = if let Some(arg) = arg.bool_output() {
			if let Some((numeric, _)) = numeric_output(&arg, NumericConversion::None) {
				Some(numeric)
			} else {
				Some(Type::Bool)
			}
		} else {
			None
		};
		output.map(|output| Self { output })
	}
}

impl IsUnaryOp for OpNeg {
	fn execute(&self, scope: &mut RuntimeScope, arg: &Node) -> Result<ExprValue> {
		let arg = arg.execute(scope)?.into_value();
		let (arg, bool) = Type::to_bool_output(&arg)?;
		if self.output == Type::Bool {
			Ok(Value::Bool(!bool).into())
		} else if let Type::Float(float) = self.output {
			let value = arg.float_value(&float, NumericConversion::None)?;
			let value = if value.as_bool() {
				FloatValue::new(0.0, float)
			} else {
				FloatValue::new(1.0, float)
			};
			Ok(Value::Float(value).into())
		} else {
			let int_type = self
				.output
				.get_int_type(NumericConversion::None)
				.expect("operator `neg` produced an invalid output")
				.clone();
			let arg = arg.int_value(&int_type, NumericConversion::None)?;
			let arg = if arg.is_zero() {
				IntValue::new(1, int_type)
			} else {
				IntValue::new(0, int_type)
			};
			Ok(Value::Int(arg?).into())
		}
	}

	fn get_type(&self) -> Type {
		self.output.clone()
	}
}
