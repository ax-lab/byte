use std::{cell::RefCell, rc::Rc};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Value {
	None,
	Null,
	Bool(bool),
	Integer(i128),
	String(String),
	LValue(Rc<RefCell<Value>>),
}

impl<'a> Default for Value {
	fn default() -> Self {
		Value::Null
	}
}

impl<'a> std::fmt::Display for Value {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Value::None => write!(f, "(none)"),
			Value::Null => write!(f, "null"),
			Value::Bool(true) => write!(f, "true"),
			Value::Bool(false) => write!(f, "false"),
			Value::Integer(val) => write!(f, "{val}"),
			Value::String(val) => write!(f, "{val}"),
			Value::LValue(val) => write!(f, "{}", val.borrow()),
		}
	}
}

impl<'a> Value {
	pub fn as_value(&self) -> Value {
		if let Value::LValue(some) = self {
			some.borrow().as_value()
		} else {
			self.clone()
		}
	}

	pub fn to_ref(&self) -> Value {
		Value::LValue(Rc::new(self.as_value().into()))
	}

	pub fn is_string(&self) -> bool {
		matches!(self, &Value::String(_))
	}

	pub fn is_integer(&self) -> bool {
		matches!(self, &Value::Integer(_))
	}

	pub fn to_bool(&self) -> bool {
		match self {
			Value::None => false,
			Value::Integer(value) => *value != 0,
			Value::String(value) => value != "",
			Value::Bool(value) => *value,
			Value::Null => false,
			Value::LValue(value) => value.borrow().to_bool(),
		}
	}

	pub fn to_string(&self) -> String {
		match self {
			Value::None => String::new(),
			Value::Integer(value) => format!("{value}"),
			Value::String(value) => value.clone(),
			Value::Bool(value) => (if value.clone() { "true" } else { "false" }).into(),
			Value::Null => Default::default(),
			Value::LValue(value) => value.borrow().to_string(),
		}
	}

	pub fn to_integer(&self) -> i128 {
		match self {
			Value::None => 0,
			Value::Integer(value) => *value,
			Value::String(_) => panic!("using string value as a number"),
			Value::Bool(value) => {
				if *value {
					1
				} else {
					0
				}
			}
			Value::Null => 0,
			Value::LValue(value) => value.borrow().to_integer(),
		}
	}

	pub fn parse_integer(&self) -> i128 {
		match self {
			Value::String(val) => val.parse().unwrap(),
			other => other.to_integer(),
		}
	}

	pub fn op_minus(&self) -> Value {
		let result = -self.to_integer();
		Value::Integer(result)
	}

	pub fn op_plus(&self) -> Value {
		let result = self.parse_integer();
		Value::Integer(result)
	}

	pub fn op_not(&self) -> Value {
		let result = !self.to_bool();
		Value::Bool(result)
	}

	pub fn op_negate(&self) -> Value {
		if self.is_integer() {
			Value::Integer(if self.to_integer() == 0 { 1 } else { 0 })
		} else {
			let result = !self.to_bool();
			Value::Bool(result)
		}
	}

	pub fn op_pre_increment(&self) -> Value {
		let value = self.to_integer() + 1;
		let value = Value::Integer(value);
		if let Value::LValue(var) = self {
			let mut var = var.borrow_mut();
			*var = value;
		}
		self.clone()
	}

	pub fn op_pre_decrement(&self) -> Value {
		let value = self.to_integer() - 1;
		let value = Value::Integer(value);
		if let Value::LValue(var) = self {
			let mut var = var.borrow_mut();
			*var = value;
		}
		self.clone()
	}

	pub fn op_pos_increment(&self) -> Value {
		let pre_value = self.as_value();
		let value = self.to_integer() + 1;
		let value = Value::Integer(value);
		if let Value::LValue(var) = self {
			let mut var = var.borrow_mut();
			*var = value;
		}
		pre_value
	}

	pub fn op_pos_decrement(&self) -> Value {
		let pre_value = self.as_value();
		let value = self.to_integer() - 1;
		let value = Value::Integer(value);
		if let Value::LValue(var) = self {
			let mut var = var.borrow_mut();
			*var = value;
		}
		pre_value
	}

	pub fn op_add(&self, b: Value) -> Value {
		if self.is_string() || b.is_string() {
			let result = format!("{}{}", self.to_string(), b.to_string());
			return Value::String(result);
		}
		Value::Integer(self.to_integer() + b.to_integer())
	}

	pub fn op_sub(&self, b: Value) -> Value {
		Value::Integer(self.to_integer() - b.to_integer())
	}

	pub fn op_mul(&self, b: Value) -> Value {
		Value::Integer(self.to_integer() * b.to_integer())
	}

	pub fn op_div(&self, b: Value) -> Value {
		Value::Integer(self.to_integer() / b.to_integer())
	}

	pub fn op_mod(&self, b: Value) -> Value {
		Value::Integer(self.to_integer() % b.to_integer())
	}

	pub fn op_equal(&self, b: Value) -> Value {
		Value::Bool(self.as_value() == b.as_value())
	}

	pub fn op_assign(&self, b: Value) -> Value {
		if let Value::LValue(var) = self {
			let mut var = var.borrow_mut();
			*var = b.as_value();
		} else {
			panic!("assigning no non-reference value");
		}
		self.clone()
	}
}
