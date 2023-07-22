use super::*;

#[derive(Debug)]
pub struct OpNot {
	output: Type,
}

impl OpNot {
	pub fn for_type(arg: &Type) -> Option<Self> {
		let arg = arg.bool_output();
		if arg.is_some() {
			Some(Self { output: Type::Bool })
		} else {
			None
		}
	}
}

impl IsUnaryOp for OpNot {
	fn execute(&self, scope: &mut RuntimeScope, arg: &Node) -> Result<ExprValue> {
		let arg = arg.execute(scope)?.into_value();
		let arg = Type::to_bool(&arg)?;
		Ok(Value::Bool(!arg).into())
	}

	fn get_type(&self) -> Type {
		self.output.clone()
	}
}
