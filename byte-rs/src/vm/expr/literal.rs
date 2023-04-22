use crate::core::str::*;
use crate::core::*;

use super::*;

#[derive(Clone, Debug)]
pub enum Literal {
	Bool(bool),
	String(String),
	Integer(u64),
}

impl IsExpr for Literal {
	fn eval(&self, rt: &mut Runtime) -> Value {
		match self {
			Literal::Bool(value) => Value::from(*value),
			Literal::String(value) => Value::from(value.clone()),
			Literal::Integer(value) => Value::any_int(*value as num::AnyInt),
		}
	}

	fn get_type(&self) -> Type {
		match self {
			Literal::Bool(..) => Type::Bool,
			Literal::String(..) => Type::String,
			Literal::Integer(..) => Type::Int(num::kind::Int::Any),
		}
	}
}

impl Value {}
