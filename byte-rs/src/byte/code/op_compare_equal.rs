use super::*;

#[derive(Debug)]
pub struct OpCompareEqual {
	output: Type,
}

impl OpCompareEqual {
	pub fn for_types(lhs: &Type, rhs: &Type) -> Option<Self> {
		let output = if lhs != rhs {
			Type::Bool
		} else {
			Type::Or(Type::Bool.into(), lhs.clone().into())
		};
		Some(Self { output })
	}
}

impl IsBinaryOp for OpCompareEqual {
	fn execute(&self, scope: &mut RuntimeScope, lhs: &Expr, rhs: &Expr) -> Result<ExprValue> {
		let lhs = lhs.execute(scope)?;
		let rhs = rhs.execute(scope)?;
		let equal = lhs.value() == rhs.value();
		if equal {
			Ok(lhs)
		} else {
			Ok(Value::from(false).into())
		}
	}

	fn get_type(&self) -> Type {
		self.output.clone()
	}
}
