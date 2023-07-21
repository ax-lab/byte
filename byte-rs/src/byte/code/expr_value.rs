use super::*;

// TODO: add Info to this

/// Wraps the result of an [`Expr`] with additional information and support
/// for references.
#[derive(Clone, Debug)]
pub enum ExprValue {
	Value(Value),
	Variable(Symbol, CodeOffset, Value),
}

impl ExprValue {
	pub fn value(&self) -> &Value {
		match self {
			ExprValue::Value(ref value) => value,
			ExprValue::Variable(.., ref value) => value,
		}
	}

	pub fn into_value(self) -> Value {
		match self {
			ExprValue::Value(value) => value,
			ExprValue::Variable(.., value) => value,
		}
	}
}

impl From<ExprValue> for Value {
	fn from(expr_value: ExprValue) -> Self {
		expr_value.value().clone()
	}
}

impl From<Value> for ExprValue {
	fn from(value: Value) -> Self {
		ExprValue::Value(value)
	}
}
