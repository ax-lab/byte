use std::collections::HashMap;

use crate::parser::{BinaryOp, Expr, ExprAtom, TernaryOp, UnaryOp};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Result {
	Ref(String),
	Value(ResultValue),
}

impl Result {
	pub fn to_value(self, map: &HashMap<String, ResultValue>) -> ResultValue {
		match self {
			Result::Ref(id) => {
				if let Some(value) = map.get(&id) {
					value.clone()
				} else {
					ResultValue::None
				}
			}
			Result::Value(value) => value,
		}
	}
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ResultValue {
	Integer(i128),
	String(String),
	Boolean(bool),
	Null,
	None,
}

impl Into<Result> for ResultValue {
	fn into(self) -> Result {
		Result::Value(self)
	}
}

impl ResultValue {
	pub fn is_string(&self) -> bool {
		matches!(self, &ResultValue::String(_))
	}

	pub fn is_integer(&self) -> bool {
		matches!(self, &ResultValue::Integer(_))
	}

	pub fn to_bool(&self) -> bool {
		match self {
			ResultValue::Integer(value) => *value != 0,
			ResultValue::String(value) => value != "",
			ResultValue::Boolean(value) => *value,
			ResultValue::None => false,
			ResultValue::Null => false,
		}
	}

	pub fn to_string(self) -> String {
		match self {
			ResultValue::Integer(value) => format!("{value}"),
			ResultValue::String(value) => value,
			ResultValue::Boolean(value) => (if value { "true" } else { "false" }).into(),
			ResultValue::None => Default::default(),
			ResultValue::Null => Default::default(),
		}
	}

	pub fn to_integer(self) -> i128 {
		match self {
			ResultValue::Integer(value) => value,
			ResultValue::String(_) => panic!("using string value as a number"),
			ResultValue::Boolean(value) => {
				if value {
					1
				} else {
					0
				}
			}
			ResultValue::None => 0,
			ResultValue::Null => 0,
		}
	}

	pub fn parse_integer(self) -> i128 {
		match self {
			ResultValue::String(val) => val.parse().unwrap(),
			other => other.to_integer(),
		}
	}
}

pub fn execute_expr(expr: &Expr, map: &mut HashMap<String, ResultValue>) -> ResultValue {
	let result = execute_expr_ref(expr, map);
	result.to_value(map)
}

fn execute_expr_ref(expr: &Expr, map: &mut HashMap<String, ResultValue>) -> Result {
	let result = match expr {
		Expr::Unary(op, expr) => {
			let expr = execute_expr(&expr, map);
			match op {
				UnaryOp::Minus => {
					let result = -expr.to_integer();
					ResultValue::Integer(result)
				}

				UnaryOp::Plus => {
					let result = expr.parse_integer();
					ResultValue::Integer(result)
				}

				UnaryOp::Not => {
					let result = !expr.to_bool();
					ResultValue::Boolean(result)
				}

				UnaryOp::Negate => {
					if expr.is_integer() {
						ResultValue::Integer(if expr.to_integer() == 0 { 1 } else { 0 })
					} else {
						let result = !expr.to_bool();
						ResultValue::Boolean(result)
					}
				}

				op => todo!("{op:?}"),
			}
		}

		Expr::Binary(BinaryOp::Assign, left, right) => {
			let left = execute_expr_ref(left, map);
			let right = execute_expr(right, map);
			if let Result::Ref(id) = left {
				map.insert(id.clone(), right);
				return Result::Ref(id);
			} else {
				panic!("cannot assign to value");
			}
		}

		Expr::Binary(op @ (BinaryOp::And | BinaryOp::Or), left, right) => {
			let left = execute_expr(left, map);
			match op {
				BinaryOp::And => {
					if left.to_bool() {
						execute_expr(right, map)
					} else {
						left
					}
				}
				BinaryOp::Or => {
					if left.to_bool() {
						left
					} else {
						execute_expr(right, map)
					}
				}
				_ => unreachable!(),
			}
		}

		Expr::Binary(op, left, right) => {
			let left = execute_expr(&left, map);
			let right = execute_expr(&right, map);

			let result = match op {
				BinaryOp::Add => {
					if left.is_string() || right.is_string() {
						let result = format!("{}{}", left.to_string(), right.to_string());
						return ResultValue::String(result).into();
					}
					left.to_integer() + right.to_integer()
				}
				BinaryOp::Sub => left.to_integer() - right.to_integer(),
				BinaryOp::Mul => left.to_integer() * right.to_integer(),
				BinaryOp::Div => left.to_integer() / right.to_integer(),
				BinaryOp::Mod => left.to_integer() % right.to_integer(),
				BinaryOp::Equal => return ResultValue::Boolean(left == right).into(),
				BinaryOp::Assign | BinaryOp::And | BinaryOp::Or => {
					unreachable!("handled explicitly")
				}
			};
			ResultValue::Integer(result)
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

		Expr::Value(ExprAtom::Integer(value)) => ResultValue::Integer(*value as i128),
		Expr::Value(ExprAtom::Literal(value)) => ResultValue::String(value.clone()),
		Expr::Value(ExprAtom::Null) => ResultValue::Null,
		Expr::Value(ExprAtom::Boolean(value)) => ResultValue::Boolean(*value),
		Expr::Value(ExprAtom::Var(id)) => return Result::Ref(id.clone()),

		expr => {
			todo!("expression {expr:?}");
		}
	};

	result.into()
}

impl std::fmt::Display for ResultValue {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			ResultValue::Integer(v) => write!(f, "{v}"),
			ResultValue::String(v) => write!(f, "{v}"),
			ResultValue::Boolean(v) => write!(f, "{}", if *v { "true" } else { "false" }),
			ResultValue::Null => write!(f, "null"),
			ResultValue::None => write!(f, "(none)"),
		}
	}
}
