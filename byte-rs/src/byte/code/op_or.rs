use super::*;

#[derive(Debug)]
pub struct OpOr {
	output: Type,
}

impl OpOr {
	pub fn for_type(lhs: &Type) -> Option<Self> {
		Self::for_types(lhs, lhs)
	}

	pub fn for_types(lhs: &Type, rhs: &Type) -> Option<Self> {
		let lhs = lhs.bool_output();
		let rhs = rhs.bool_output();
		let lhs = if let Some(lhs) = lhs {
			lhs
		} else {
			return None;
		};
		let rhs = if let Some(rhs) = rhs {
			rhs
		} else {
			return None;
		};

		let output = Type::merge_for_upcast(lhs, rhs);
		Some(Self { output })
	}
}

impl IsBinaryOp for OpOr {
	fn execute(&self, scope: &mut RuntimeScope, lhs: &Expr, rhs: &Expr) -> Result<ExprValue> {
		let lhs = lhs.execute(scope)?.into_value();
		let (lhs, bool) = Type::to_bool_output(&lhs)?;
		if bool {
			Ok(lhs.into())
		} else {
			let rhs = rhs.execute(scope)?.into_value();
			let (rhs, _) = Type::to_bool_output(&rhs)?;
			Ok(rhs.into())
		}
	}

	fn get_type(&self) -> Type {
		self.output.clone()
	}
}
