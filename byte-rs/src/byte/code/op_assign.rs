use super::*;

#[derive(Debug)]
pub struct OpAssign(pub Type);

impl IsBinaryOp for OpAssign {
	fn execute(&self, scope: &mut RuntimeScope, lhs: &Expr, rhs: &Expr) -> Result<ExprValue> {
		let typ = rhs.get_type();
		let rhs = rhs.execute(scope)?;
		let lhs = lhs.execute(scope)?;
		match lhs {
			ExprValue::Value(..) => {
				// TODO: runtime should have access to the source of expressions
				let error = format!("cannot assign `{typ}` to non-reference");
				let error = Errors::from(error, Span::default());
				Err(error)
			}
			ExprValue::Variable(name, index, ..) => {
				scope.set(name.clone(), index, rhs.value().clone());
				Ok(ExprValue::Variable(name, index, rhs.into_value()))
			}
		}
	}

	fn get_type(&self) -> Type {
		self.0.clone()
	}
}
