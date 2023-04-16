use crate::core::str::*;

use super::strings::StrValue;
use super::*;

#[derive(Clone, Debug)]
pub enum Literal {
	Bool(bool),
	String(Str),
	Integer(u64),
}

impl IsExpr for Literal {
	fn eval(&self, rt: &mut Runtime) -> Value {
		match self {
			Literal::Bool(value) => Value::bool(*value),
			Literal::String(value) => StrValue::new(value.clone()),
			Literal::Integer(value) => ValueInt::any(*value),
		}
	}

	fn get_type(&self) -> Type {
		match self {
			Literal::Bool(..) => Type::Bool,
			Literal::String(..) => StrValue::get_type(),
			Literal::Integer(..) => Type::Int(TypeInt::Any),
		}
	}
}

impl Value {}