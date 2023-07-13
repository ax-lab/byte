use super::*;

#[derive(Debug)]
pub struct OpMul {
	output: Type,
}

impl OpMul {
	pub fn for_type(lhs: &Type) -> Option<Self> {
		Self::for_types(lhs, lhs)
	}

	pub fn for_types(lhs: &Type, rhs: &Type) -> Option<Self> {
		arithmetic_output(lhs, rhs).map(|output| Self { output })
	}
}

impl IsBinaryOp for OpMul {
	fn execute(&self, scope: &mut RuntimeScope, lhs: &Expr, rhs: &Expr) -> Result<ExprValue> {
		const CONVERSION: NumericConversion = ARITHMETIC_CONVERSION;
		let lhs = lhs.execute(scope)?.into_value();
		let rhs = rhs.execute(scope)?.into_value();
		if let Type::Float(float_type) = self.output {
			let lhs = lhs.float_value(&float_type, CONVERSION)?;
			let rhs = rhs.float_value(&float_type, CONVERSION)?;
			let value = lhs.as_f64() * rhs.as_f64();
			let value = FloatValue::new(value, float_type);
			Ok(Value::Float(value).into())
		} else {
			let int_type = self
				.output
				.get_int_type(CONVERSION)
				.expect("operator `mul` produced an invalid output")
				.clone();
			let lhs = lhs.int_value(&int_type, CONVERSION)?;
			let rhs = rhs.int_value(&int_type, CONVERSION)?;
			if int_type.signed() {
				let (result, overflow) = lhs.signed().overflowing_mul(rhs.signed());
				if overflow {
					err!("integer overflow for mul of {int_type}")
				} else {
					let value = IntValue::new_signed(result, int_type)?;
					Ok(Value::Int(value).into())
				}
			} else {
				let (result, overflow) = lhs.unsigned().overflowing_mul(rhs.unsigned());
				if overflow {
					err!("integer overflow for mul of {int_type}")
				} else {
					let value = IntValue::new(result, int_type)?;
					Ok(Value::Int(value).into())
				}
			}
		}
	}

	fn get_type(&self) -> Type {
		self.output.clone()
	}
}
