//! Expression for plain values, such as literal numbers and strings.

use super::*;

// TODO: merge into main values.rs

//====================================================================================================================//
// Builtin Literal Values
//====================================================================================================================//

#[derive(Clone, Debug)]
pub enum ValueExpr {
	Bool(bool),
	Str(StrValue),
	Int(IntValue),
}

impl ValueExpr {
	pub fn get_type(&self) -> Type {
		match self {
			ValueExpr::Bool(..) => Type::Bool,
			ValueExpr::Str(..) => Type::String,
			ValueExpr::Int(int) => Type::Int(int.get_type()),
		}
	}

	pub fn execute(&self, scope: &mut RuntimeScope) -> Result<Value> {
		let _ = scope;
		match self {
			ValueExpr::Bool(value) => Ok(Value::from(*value)),
			ValueExpr::Str(value) => Ok(Value::from(value.to_string())),
			ValueExpr::Int(value) => Ok(Value::from(value.clone())),
		}
	}
}

#[derive(Clone, Debug)]
pub struct StrValue(Arc<String>);

impl StrValue {
	pub fn new<T: Into<String>>(str: T) -> Self {
		let str = str.into();
		Self(Arc::new(str))
	}

	pub fn new_from_arc(str: Arc<String>) -> Self {
		Self(str)
	}

	pub fn as_str(&self) -> &str {
		self.0.as_str()
	}
}

impl Display for StrValue {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.as_str())
	}
}

impl<T: Into<String>> From<T> for StrValue {
	fn from(value: T) -> Self {
		StrValue(value.into().into())
	}
}
