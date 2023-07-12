use super::*;

const MINUS_CONVERSION: NumericConversion = NumericConversion::Parse;

#[derive(Debug)]
pub struct OpMinus {
	output: Type,
}

impl OpMinus {
	pub fn for_type(arg: &Type) -> Option<Self> {
		numeric_output(arg, MINUS_CONVERSION)
			.and_then(|(output, signed)| if signed { Some(Self { output }) } else { None })
	}
}

impl IsUnaryOp for OpMinus {
	fn execute(&self, scope: &mut RuntimeScope, arg: &Expr) -> Result<ExprValue> {
		let arg = arg.execute(scope)?.value();
		if let Type::Float(float_type) = self.output {
			let arg = arg.float_value(&float_type, MINUS_CONVERSION)?;
			let value = -arg.as_f64();
			let value = FloatValue::new(value, float_type);
			Ok(Value::Float(value).into())
		} else {
			let int_type = self
				.output
				.get_int_type(MINUS_CONVERSION)
				.expect("operator `minus` produced an invalid output")
				.clone();
			let arg = arg.int_value(&int_type, MINUS_CONVERSION)?;
			let arg = -arg.signed();
			Ok(Value::Int(IntValue::new_signed(arg, int_type)?).into())
		}
	}

	fn get_type(&self) -> Type {
		self.output.clone()
	}
}
