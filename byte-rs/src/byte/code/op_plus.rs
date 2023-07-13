use super::*;

const PLUS_CONVERSION: NumericConversion = NumericConversion::Parse;

#[derive(Debug)]
pub struct OpPlus {
	output: Type,
}

impl OpPlus {
	pub fn for_type(arg: &Type) -> Option<Self> {
		numeric_output(arg, PLUS_CONVERSION).map(|(output, _)| Self { output })
	}
}

impl IsUnaryOp for OpPlus {
	fn execute(&self, scope: &mut RuntimeScope, arg: &Expr) -> Result<ExprValue> {
		let arg = arg.execute(scope)?.into_value();
		if let Type::Float(float_type) = self.output {
			let arg = arg.float_value(&float_type, PLUS_CONVERSION)?;
			let value = arg.as_f64();
			let value = FloatValue::new(value, float_type);
			Ok(Value::Float(value).into())
		} else {
			let int_type = self
				.output
				.get_int_type(PLUS_CONVERSION)
				.expect("operator `plus` produced an invalid output")
				.clone();
			let arg = arg.int_value(&int_type, PLUS_CONVERSION)?;
			Ok(Value::Int(arg).into())
		}
	}

	fn get_type(&self) -> Type {
		self.output.clone()
	}
}
