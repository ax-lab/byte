use super::*;

pub mod scope;

pub use scope::*;

impl Expr {
	pub fn eval(&self, scope: &mut Scope) -> Result<Value> {
		match self {
			Expr::Value(value) => value.eval(scope),
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
		let value = self.data;
		let value = match self.kind {
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

#[cfg(testX)]
mod tests {
	use super::*;
	use crate::code::*;

	#[test]
	fn basic_eval() -> Result<()> {
		let a: TypedExpr<I32> = TypedExpr::Value(2);
		let b: TypedExpr<I32> = TypedExpr::Value(2);
		let expr = TypedExpr::Binary(BinaryOp::new(IntAdd), OpValue::new(a), OpValue::new(b));
		let expr = TypedExpr::Unary(UnaryOp::new(IntMinus), OpValue::new(expr));

		let mut scope = Scope::new();
		let result = scope.eval(&expr)?;
		assert_eq!(result, Value::from(-4));

		Ok(())
	}
}
