use super::*;

pub mod scope;

pub use scope::*;

pub trait WithEval {
	fn eval(&self, scope: &mut Scope) -> Result<Value>;
}

impl Scope {
	pub fn eval<T: IsValue + std::fmt::Debug + ?Sized>(&mut self, expr: &T) -> Result<Value> {
		if let Some(expr) = get_trait!(expr, WithEval) {
			expr.eval(self)
		} else {
			Err(Errors::from(format!("expression does not support eval: {expr:?}")))
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::code::*;

	#[test]
	fn basic_eval() -> Result<()> {
		let a: Expr<I32> = Expr::Value(2);
		let b: Expr<I32> = Expr::Value(2);
		let expr = Expr::Binary(BinaryOp::new(IntAdd), OpValue::new(a), OpValue::new(b));
		let expr = Expr::Unary(UnaryOp::new(IntMinus), OpValue::new(expr));

		let mut scope = Scope::new();
		let result = scope.eval(&expr)?;
		assert_eq!(result, Value::from(-4));

		Ok(())
	}
}
