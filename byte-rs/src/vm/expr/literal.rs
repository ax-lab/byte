use crate::core::str::*;

use super::strings::StringValue;
use super::*;

#[derive(Clone, Debug)]
pub enum Literal {
	Bool(bool),
	String(Str),
	Integer(Str),
}

impl IsExpr for Literal {
	fn eval(&self, rt: &mut Runtime) -> Value {
		match self {
			Literal::Bool(value) => Value::bool(*value),
			Literal::String(value) => StringValue::new(value.clone()),
			Literal::Integer(value) => {
				let value: u64 = value.as_str().parse().expect("invalid integer literal");
				ValueInt::any(value)
			}
		}
	}
}

impl Value {}
