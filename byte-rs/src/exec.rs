use std::collections::HashMap;

use crate::parser::{BinaryOp, Expr, TernaryOp, UnaryOp, Value};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ResultCell {
	Ref(String),
	Value(Result),
}

impl ResultCell {
	pub fn to_value(self, map: &HashMap<String, Result>) -> Result {
		match self {
			ResultCell::Ref(id) => {
				if let Some(value) = map.get(&id) {
					value.clone()
				} else {
					Result::None
				}
			}
			ResultCell::Value(value) => value,
		}
	}
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Result {
	Integer(i64),
	String(String),
	Boolean(bool),
	Null,
	None,
}

impl Into<ResultCell> for Result {
	fn into(self) -> ResultCell {
		ResultCell::Value(self)
	}
}

impl Result {
	pub fn is_string(&self) -> bool {
		matches!(self, &Result::String(_))
	}

	pub fn is_integer(&self) -> bool {
		matches!(self, &Result::Integer(_))
	}

	pub fn to_bool(self) -> bool {
		match self {
			Result::Integer(value) => value != 0,
			Result::String(value) => value != "",
			Result::Boolean(value) => value,
			Result::None => false,
			Result::Null => false,
		}
	}

	pub fn to_string(self) -> String {
		match self {
			Result::Integer(value) => format!("{value}"),
			Result::String(value) => value,
			Result::Boolean(value) => (if value { "true" } else { "false" }).into(),
			Result::None => Default::default(),
			Result::Null => Default::default(),
		}
	}

	pub fn to_integer(self) -> i64 {
		match self {
			Result::Integer(value) => value,
			Result::String(_) => panic!("using string value as a number"),
			Result::Boolean(value) => {
				if value {
					1
				} else {
					0
				}
			}
			Result::None => 0,
			Result::Null => 0,
		}
	}

	pub fn parse_integer(self) -> i64 {
		match self {
			Result::String(val) => val.parse().unwrap(),
			other => other.to_integer(),
		}
	}
}

pub fn execute_expr(expr: &Expr, map: &mut HashMap<String, Result>) -> Result {
	let result = execute_expr_ref(expr, map);
	result.to_value(map)
}

fn execute_expr_ref(expr: &Expr, map: &mut HashMap<String, Result>) -> ResultCell {
	let result = match expr {
		Expr::Unary(op, expr) => {
			let expr = execute_expr(&expr, map);
			match op {
				UnaryOp::Minus => {
					let result = -expr.to_integer();
					Result::Integer(result)
				}

				UnaryOp::Plus => {
					let result = expr.parse_integer();
					Result::Integer(result)
				}

				UnaryOp::Not => {
					let result = !expr.to_bool();
					Result::Boolean(result)
				}

				UnaryOp::Negate => {
					if expr.is_integer() {
						Result::Integer(if expr.to_integer() == 0 { 1 } else { 0 })
					} else {
						let result = !expr.to_bool();
						Result::Boolean(result)
					}
				}

				op => todo!("{op:?}"),
			}
		}

		Expr::Binary(BinaryOp::Assign, left, right) => {
			let left = execute_expr_ref(left, map);
			let right = execute_expr(right, map);
			if let ResultCell::Ref(id) = left {
				map.insert(id.clone(), right);
				return ResultCell::Ref(id);
			} else {
				panic!("cannot assign to value");
			}
		}

		Expr::Binary(op, left, right) => {
			let left = execute_expr(&left, map);
			let right = execute_expr(&right, map);

			let result = match op {
				BinaryOp::Add => {
					if left.is_string() || right.is_string() {
						let result = format!("{}{}", left.to_string(), right.to_string());
						return Result::String(result).into();
					}
					left.to_integer() + right.to_integer()
				}
				BinaryOp::Sub => left.to_integer() - right.to_integer(),
				BinaryOp::Mul => left.to_integer() * right.to_integer(),
				BinaryOp::Div => left.to_integer() / right.to_integer(),
				BinaryOp::Mod => left.to_integer() % right.to_integer(),
				BinaryOp::Equal => return Result::Boolean(left == right).into(),
				BinaryOp::Assign => unreachable!("assign is handled explicitly"),
			};
			Result::Integer(result)
		}

		Expr::Ternary(TernaryOp::Condition, cond, if_true, if_false) => {
			let cond = execute_expr(&cond, map);
			let result = if cond.to_bool() {
				execute_expr(&if_true, map)
			} else {
				execute_expr(&if_false, map)
			};
			result
		}

		Expr::Value(Value::Integer(value)) => Result::Integer(value.parse().unwrap()),
		Expr::Value(Value::Literal(value)) => Result::String(value.clone()),
		Expr::Value(Value::Null) => Result::Null,
		Expr::Value(Value::Boolean(value)) => Result::Boolean(*value),
		Expr::Value(Value::Var(id)) => return ResultCell::Ref(id.clone()),

		expr => {
			todo!("expression {expr:?}");
		}
	};

	result.into()
}

impl std::fmt::Display for Result {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Result::Integer(v) => write!(f, "{v}"),
			Result::String(v) => write!(f, "{v}"),
			Result::Boolean(v) => write!(f, "{}", if *v { "true" } else { "false" }),
			Result::Null => write!(f, "null"),
			Result::None => write!(f, "(none)"),
		}
	}
}
