use super::*;

pub mod scope;

pub use scope::*;

impl Expr {
	pub fn eval(&self, scope: &mut Scope) -> Result<Value> {
		match self {
			Expr::Value(value) => value.eval(scope),
			Expr::Binary(op, lhs, rhs) => {
				let lhs = lhs.get().eval(scope)?;
				let rhs = rhs.get().eval(scope)?;
				op.get().eval(lhs, rhs)
			}
		}
	}
}

impl ValueExpr {
	pub fn eval(&self, scope: &mut Scope) -> Result<Value> {
		match self {
			ValueExpr::Unit => Ok(Value::from(())),
			ValueExpr::Never => Err(Errors::from("evaluated to never value")),
			ValueExpr::Bool(value) => Ok(Value::from(*value)),
			ValueExpr::Str(value) => Ok(Value::from(value.get())),
			ValueExpr::Int(value) => value.eval(scope),
			ValueExpr::Float(_) => todo!(),
		}
	}
}

impl IntValue {
	pub fn eval(&self, scope: &mut Scope) -> Result<Value> {
		let _ = scope;
		let value = self.value();
		let value = match self.get_type() {
			IntType::I8 => Value::from(value as i8),
			IntType::U8 => Value::from(value as u8),
			IntType::I16 => Value::from(value as i16),
			IntType::U16 => Value::from(value as u16),
			IntType::I32 => Value::from(value as i32),
			IntType::U32 => Value::from(value as u32),
			IntType::I64 => Value::from(value as i64),
			IntType::U64 => Value::from(value as u64),
			IntType::I128 => Value::from(value as i128),
			IntType::U128 => Value::from(value as u128),
		};
		Ok(value)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::code::*;

	#[test]
	fn basic_eval() -> Result<()> {
		let compiler = Compiler::new();
		let a = Expr::Value(ValueExpr::Int(IntValue::new(2, IntType::I64)));
		let b = Expr::Value(ValueExpr::Int(IntValue::new(3, IntType::I64)));
		let op = BinaryOp::from(OpAdd::for_type(&a.get_type()).unwrap());

		let a = compiler.store(a);
		let b = compiler.store(b);
		let expr = Expr::Binary(op, a, b);

		let mut scope = Scope::new();
		let result = expr.eval(&mut scope)?;
		assert_eq!(result, Value::from(5i64));

		Ok(())
	}
}
