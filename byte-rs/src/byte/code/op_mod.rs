use super::*;

#[derive(Debug)]
pub struct OpMod {
	output: Type,
}

impl OpMod {
	pub fn for_type(lhs: &Type) -> Option<Self> {
		Self::for_types(lhs, lhs)
	}

	pub fn for_types(lhs: &Type, rhs: &Type) -> Option<Self> {
		arithmetic_output(lhs, rhs).map(|output| Self { output })
	}
}

impl IsBinaryOp for OpMod {
	fn execute(&self, scope: &mut RuntimeScope, lhs: &Expr, rhs: &Expr) -> Result<ExprValue> {
		const CONVERSION: NumericConversion = ARITHMETIC_CONVERSION;
		let lhs = lhs.execute(scope)?.value();
		let rhs = rhs.execute(scope)?.value();
		let int_type = self
			.output
			.get_int_type(CONVERSION)
			.expect("operator `mod` produced an invalid output")
			.clone();
		let lhs = lhs.int_value(&int_type, CONVERSION)?;
		let rhs = rhs.int_value(&int_type, CONVERSION)?;
		if rhs.is_zero() {
			err!("division by zero in mod of {int_type}")
		} else if int_type.signed() {
			let result = lhs.signed() % rhs.signed();
			let value = IntValue::new_signed(result, int_type)?;
			Ok(Value::Int(value).into())
		} else {
			let result = lhs.unsigned() % rhs.unsigned();
			let value = IntValue::new(result, int_type)?;
			Ok(Value::Int(value).into())
		}
	}

	fn get_type(&self) -> Type {
		self.output.clone()
	}
}
