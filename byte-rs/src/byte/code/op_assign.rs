use super::*;

#[derive(Debug)]
pub struct OpAssign(pub Type);

impl IsBinaryOp for OpAssign {
	fn execute(&self, scope: &mut RuntimeScope, lhs: &Node, rhs: &Node) -> Result<ExprValue> {
		let typ = rhs.get_type()?;
		let rhs_val = rhs.execute(scope)?;
		let lhs_val = lhs.execute(scope)?;
		match lhs_val {
			ExprValue::Value(..) => {
				let error = format!("cannot assign `{typ}` to non-reference");
				let error = Errors::from(error, lhs.span().clone());
				Err(error)
			}
			ExprValue::Variable(name, offset, ..) => {
				scope.set(name.clone(), offset, rhs_val.value().clone());
				Ok(ExprValue::Variable(name, offset, rhs_val.into_value()))
			}
		}
	}

	fn get_type(&self) -> Type {
		self.0.clone()
	}
}
