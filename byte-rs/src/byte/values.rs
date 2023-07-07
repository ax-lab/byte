use super::*;

pub mod int;
pub mod types;

pub use int::*;
pub use types::*;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Value {
	Unit,
	Never,
	Null,
	Bool(bool),
	Int(IntValue),
	String(Arc<String>),
}

impl Value {
	pub fn is_unit(&self) -> bool {
		matches!(self, Value::Unit)
	}

	pub fn is_string(&self) -> bool {
		matches!(self, Value::String(..))
	}

	pub fn string(&self) -> Result<StrValue> {
		let value = match self {
			Value::Unit => "".into(),
			Value::Never => return err!("cannot convert never value to string"),
			Value::Null => "(null)".into(),
			Value::Bool(value) => (if *value { "true" } else { "false " }).into(),
			Value::Int(value) => value.to_string().into(),
			Value::String(value) => StrValue::new_from_arc(value.clone()),
		};
		Ok(value)
	}

	pub fn bool(&self) -> Result<bool> {
		Type::to_bool(self)
	}

	pub fn int_value(&self, kind: &IntType, conversion: NumericConversion) -> Result<IntValue> {
		let output = match self {
			Value::Unit => None,
			Value::Never => None,
			Value::Null => None,
			Value::Bool(value) if conversion >= NumericConversion::BoolToInt => {
				let output = IntValue::new(if *value { 1 } else { 0 }, kind.clone())?;
				Some(output)
			}
			Value::Int(int) => Some(int.cast_to(kind)?),
			Value::String(value) if conversion >= NumericConversion::Parse => {
				let value: DefaultInt = match value.as_str().parse() {
					Ok(value) => value,
					Err(err) => return err!("cannot parse string as {kind}: {err}"),
				};
				Some(IntValue::new(value as u128, DEFAULT_INT)?)
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
			Value::String(..) => Type::String,
		}
	}
}

impl Display for Value {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "{self:?}") // TODO: implement proper
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
