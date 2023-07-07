use super::*;

#[derive(Debug)]
pub struct OpDiv {
	output: Type,
}

impl OpDiv {
	pub fn for_type(lhs: &Type) -> Option<Self> {
		Self::for_types(lhs, lhs)
	}

	pub fn for_types(lhs: &Type, rhs: &Type) -> Option<Self> {
		arithmetic_output(lhs, rhs).map(|output| Self { output })
	}
}

impl IsBinaryOp for OpDiv {
	fn execute(&self, scope: &mut RuntimeScope, lhs: &Expr, rhs: &Expr) -> Result<ExprValue> {
		const CONVERSION: NumericConversion = ARITHMETIC_CONVERSION;
		let lhs = lhs.execute(scope)?.value();
		let rhs = rhs.execute(scope)?.value();
		let int_type = self
			.output
			.get_int_type(CONVERSION)
			.expect("operator `div` produced an invalid output")
			.clone();
		let lhs = lhs.int_value(&int_type, CONVERSION)?;
		let rhs = rhs.int_value(&int_type, CONVERSION)?;
		if rhs.is_zero() {
			err!("division by zero of {int_type}")
		} else if int_type.signed() {
			let (result, overflow) = lhs.signed().overflowing_div(rhs.signed());
			if overflow {
				err!("integer overflow for div of {int_type}")
			} else {
				let value = IntValue::new_signed(result, int_type)?;
				Ok(Value::Int(value).into())
			}
		} else {
			let (result, overflow) = lhs.unsigned().overflowing_div(rhs.unsigned());
			if overflow {
				err!("integer overflow for div of {int_type}")
			} else {
				let value = IntValue::new(result, int_type)?;
				Ok(Value::Int(value).into())
			}
		}
	}

	fn get_type(&self) -> Type {
		self.output.clone()
	}
}
