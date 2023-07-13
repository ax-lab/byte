use super::*;

pub mod float;
pub mod int;
pub mod string;
pub mod types;

pub use float::*;
pub use int::*;
pub use string::*;
pub use types::*;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Value {
	Unit,
	Never,
	Null,
	Bool(bool),
	Int(IntValue),
	Float(FloatValue),
	String(Arc<String>),
}

impl Value {
	pub fn is_unit(&self) -> bool {
		matches!(self, Value::Unit)
	}

	pub fn is_string(&self) -> bool {
		matches!(self, Value::String(..))
	}

	pub fn string(&self) -> Result<StringValue> {
		let value = match self {
			Value::Unit => "".into(),
			Value::Never => return err!("cannot convert never value to string"),
			Value::Null => "(null)".into(),
			Value::Bool(value) => (if *value { "true" } else { "false " }).into(),
			Value::Int(value) => value.to_string().into(),
			Value::Float(value) => value.to_string().into(),
			Value::String(value) => StringValue::new_from_arc(value.clone()),
		};
		Ok(value)
	}

	pub fn bool(&self) -> Result<bool> {
		Type::to_bool(self)
	}

	pub fn cast_to(&self, to: &Type, conversion: NumericConversion) -> Result<Value> {
		if to == &self.get_type() {
			Ok(self.clone())
		} else {
			let output = match to {
				Type::Unit => None,
				Type::Null => None,
				Type::Never => None,
				Type::Bool => {
					let value = self.bool()?;
					Some(Value::Bool(value))
				}
				Type::String => {
					let value = self.to_string();
					Some(Value::String(value.into()))
				}
				Type::Int(kind) => {
					let value = self.int_value(kind, conversion)?;
					Some(Value::Int(value))
				}
				Type::Float(kind) => {
					let value = self.float_value(kind, conversion)?;
					Some(Value::Float(value))
				}
				Type::Or(a, b) => {
					return self
						.cast_to(a, conversion)
						.or_else(|mut err| match self.cast_to(b, conversion) {
							Ok(value) => Ok(value),
							Err(errors) => {
								err.append(&errors);
								Err(err)
							}
						});
				}
				Type::Ref(inner) => return self.cast_to(inner, conversion),
			};
			if let Some(output) = output {
				Ok(output)
			} else {
				Err(Errors::from(
					format!("conversion from {} to {to} is not supported", self.get_type()),
					Span::default(),
				))
			}
		}
	}

	pub fn int_value(&self, kind: &IntType, conversion: NumericConversion) -> Result<IntValue> {
		let output = match self {
			Value::Unit => None,
			Value::Never => None,
			Value::Null => None,
			Value::Bool(value) if conversion >= NumericConversion::FromBool => {
				let output = IntValue::new(if *value { 1 } else { 0 }, kind.clone())?;
				Some(output)
			}
			Value::Int(value) => Some(value.cast_to(kind)?),
			Value::Float(value) => Some(IntValue::new_signed(value.as_f64() as i128, kind.clone())?),
			Value::String(value) if conversion >= NumericConversion::Parse => {
				let value: DefaultInt = match value.as_str().parse() {
					Ok(value) => value,
					Err(err) => return err!("cannot parse string as {kind}: {err}"),
				};
				Some(IntValue::new_signed(value as i128, kind.clone())?)
			}
			_ => None,
		};
		if let Some(output) = output {
			Ok(output)
		} else {
			err!("cannot convert {} to {kind}", self.get_type())
		}
	}

	pub fn float_value(&self, kind: &FloatType, conversion: NumericConversion) -> Result<FloatValue> {
		let output = match self {
			Value::Unit => None,
			Value::Never => None,
			Value::Null => None,
			Value::Bool(value) if conversion >= NumericConversion::FromBool => {
				let output = FloatValue::new(if *value { 1.0 } else { 0.0 }, kind.clone());
				Some(output)
			}
			Value::Float(value) => Some(value.clone()),
			Value::Int(value) => Some(if value.get_type().signed() {
				FloatValue::new(value.signed() as f64, kind.clone())
			} else {
				FloatValue::new(value.unsigned() as f64, kind.clone())
			}),
			Value::String(value) if conversion >= NumericConversion::Parse => {
				let value: f64 = match value.as_str().parse() {
					Ok(value) => value,
					Err(err) => return err!("cannot parse string as {kind}: {err}"),
				};
				Some(FloatValue::new(value as f64, kind.clone()))
			}
			_ => None,
		};
		if let Some(output) = output {
			Ok(output)
		} else {
			err!("cannot convert {} to {kind}", self.get_type())
		}
	}

	pub fn get_type(&self) -> Type {
		match self {
			Value::Unit => Type::Unit,
			Value::Never => Type::Never,
			Value::Null => Type::Null,
			Value::Bool(..) => Type::Bool,
			Value::Int(int) => Type::Int(int.get_type()),
			Value::Float(float) => Type::Float(float.get_type()),
			Value::String(..) => Type::String,
		}
	}
}

impl Display for Value {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		match self {
			Value::Unit => write!(f, "()"),
			Value::Never => write!(f, "(!)"),
			Value::Null => write!(f, "null"),
			Value::Bool(v) => write!(f, "{}", if *v { "true" } else { "false" }),
			Value::Int(v) => write!(f, "{v}"),
			Value::Float(v) => write!(f, "{v}"),
			Value::String(v) => write!(f, "{v}"),
		}
	}
}

impl From<()> for Value {
	fn from(_: ()) -> Self {
		Value::Unit
	}
}

impl From<bool> for Value {
	fn from(value: bool) -> Self {
		Value::Bool(value)
	}
}

impl<T: IsIntType> From<T> for Value {
	fn from(value: T) -> Self {
		Value::Int(value.to_int())
	}
}

impl From<IntValue> for Value {
	fn from(value: IntValue) -> Self {
		Value::Int(value)
	}
}

impl From<f32> for Value {
	fn from(value: f32) -> Self {
		Value::Float(FloatValue::new32(value))
	}
}

impl From<f64> for Value {
	fn from(value: f64) -> Self {
		Value::Float(FloatValue::new(value, FloatType::F64))
	}
}

impl From<FloatValue> for Value {
	fn from(value: FloatValue) -> Self {
		Value::Float(value)
	}
}

impl From<String> for Value {
	fn from(value: String) -> Self {
		Value::String(value.into())
	}
}

impl From<&str> for Value {
	fn from(value: &str) -> Self {
		Value::String(value.to_string().into())
	}
}
