use super::*;

#[derive(Debug)]
pub struct OpCompareEqual {
	output: Type,
}

impl OpCompareEqual {
	pub fn for_types(lhs: &Type, rhs: &Type) -> Option<Self> {
		let _ = (lhs, rhs);
		let output = Type::Bool;
		Some(Self { output })
	}
}

impl IsBinaryOp for OpCompareEqual {
	fn execute(&self, scope: &mut RuntimeScope, lhs: &Expr, rhs: &Expr) -> Result<ExprValue> {
		let lhs = lhs.execute(scope)?;
		let rhs = rhs.execute(scope)?;
		let equal = lhs.value() == rhs.value();
		Ok(Value::from(equal).into())
	}

	fn get_type(&self) -> Type {
		self.output.clone()
	}
}
