use crate::core::str::*;
use crate::core::*;

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
			Literal::Integer(value) => num::Int::any(*value as num::AnyInt),
		}
	}

	fn get_type(&self) -> Type {
		match self {
			Literal::Bool(..) => Type::Bool,
			Literal::String(..) => StrValue::get_type(),
			Literal::Integer(..) => Type::Int(kind::Int::Any),
		}
	}
}

impl Value {}
