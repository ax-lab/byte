use super::*;

#[derive(Debug)]
pub struct OpAssign(pub Type);

has_traits!(OpAssign: IsBinaryOp);

impl IsBinaryOp for OpAssign {
	fn execute(&self, scope: &mut RuntimeScope, lhs: &Expr, rhs: &Expr) -> Result<ExprValue> {
		let typ = rhs.get_type();
		let rhs = rhs.execute(scope)?.value();
		let lhs = lhs.execute(scope)?;
		match lhs {
			ExprValue::Value(..) => {
				// TODO: runtime should have access to the source of expressions
				let error = format!("cannot assign `{typ}` to non-reference");
				let error = Errors::from(error);
				Err(error)
			}
			ExprValue::Variable(name, index, ..) => {
				scope.set(name.clone(), index, rhs.clone());
				Ok(ExprValue::Variable(name, index, rhs))
			}
		}
	}

	fn get_type(&self) -> Type {
		self.0.clone()
	}
}
