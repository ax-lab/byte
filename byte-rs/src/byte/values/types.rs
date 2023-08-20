use super::*;

/// Enumeration of builtin types.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum Type {
	Unknown,
	Any,
	Unit,
	Null,
	Never,
	Bool,
	String,
	Int(IntType),
	Float(FloatType),
	Or(Arc<Type>, Arc<Type>),
	Ref(Arc<Type>),
}

pub enum LookupKey {
	UnaryOp(UnaryOp),
	BinaryOp(BinaryOp),
	Member(Symbol),
}

pub struct Func;

impl Func {
	pub fn get_type(&self) -> Type {
		todo!()
	}
}

impl Type {
	pub fn or(a: Self, b: Self) -> Self {
		let _ = (a, b);
		todo!()
	}

	pub fn and(a: Self, b: Self) -> Self {
		let _ = (a, b);
		todo!()
	}

	pub fn lookup(&self, key: &LookupKey, args: Vec<Type>) -> Result<Vec<Func>> {
		let _ = (key, args);
		todo!()
	}

	/// Merge two types into an upper base type that encompasses both.
	///
	/// If the types are unrelated, this will return [`Type::Or`] instead.
	pub fn merge_for_upcast(a: Self, b: Self) -> Self {
		if a == b {
			a
		} else if a == Type::Any || b == Type::Any {
			Type::Any
		} else if a == Type::Unknown || b == Type::Unknown {
			Type::Unknown
		} else {
			let a = match a {
				Type::Any | Type::Unknown => unreachable!(),
				Type::Unit => a,
				Type::Null => a,
				Type::Never => return b,
				Type::Bool => {
					if let Some(b) = b.get_int_type(NumericConversion::None) {
						let merged = IntType::merge_for_upcast(IntType::U8, b);
						return Type::Int(merged);
					} else {
						a
					}
				}
				Type::String => a,
				Type::Int(a) => {
					let a = if let Some(b) = b.get_int_type(NumericConversion::FromBool) {
						let merged = IntType::merge_for_upcast(a, b);
						return Type::Int(merged);
					} else {
						a
					};
					Type::Int(a)
				}
				Type::Float(a) => {
					let a = if let Some(b) = b.get_float_type(NumericConversion::FromBool) {
						let merged = FloatType::merge_for_upcast(a, b);
						return Type::Float(merged);
					} else {
						a
					};
					Type::Float(a)
				}
				Type::Or(..) => a,
				Type::Ref(..) => a,
			};
			Type::Or(a.into(), b.into())
		}
	}

	pub fn name(&self) -> StringValue {
		match self {
			Type::Unknown => "unknown".into(),
			Type::Any => "any".into(),
			Type::Unit => "unit".into(),
			Type::Null => "null".into(),
			Type::Never => "never".into(),
			Type::Bool => "bool".into(),
			Type::String => "string".into(),
			Type::Int(typ) => typ.name(),
			Type::Float(typ) => typ.name(),
			Type::Or(a, b) => format!("{} | {}", a.name(), b.name()).into(),
			Type::Ref(a) => format!("&({})", a.name()).into(),
		}
	}

	/// Return the actual type for the a value, disregarding reference types.
	pub fn value(&self) -> &Type {
		match self {
			Type::Ref(val) => val.as_ref(),
			_ => self,
		}
	}

	pub fn is_string(&self) -> bool {
		matches!(self.value(), Type::String)
	}

	pub fn is_float(&self) -> bool {
		matches!(self.value(), Type::Float(..))
	}

	pub fn is_bool(&self) -> bool {
		matches!(self.value(), Type::Bool)
	}

	pub fn is_int(&self) -> bool {
		matches!(self.value(), Type::Int(..))
	}

	pub fn get_numeric_type(&self, convert: NumericConversion) -> Option<Type> {
		match self.value() {
			Type::Int(..) => Some(self.clone()),
			Type::Float(..) => Some(self.clone()),
			Type::Bool if convert >= NumericConversion::FromBool => Some(Type::Int(IntType::U8)),
			Type::String if convert >= NumericConversion::Parse => Some(Type::Float(FloatType::F64)),
			_ => None,
		}
	}

	pub fn get_int_type(&self, convert: NumericConversion) -> Option<IntType> {
		match self.value() {
			Type::Int(int) => Some(int.clone()),
			Type::Bool if convert >= NumericConversion::FromBool => Some(IntType::U8),
			Type::String if convert >= NumericConversion::Parse => Some(DEFAULT_INT),
			_ => None,
		}
	}

	pub fn get_float_type(&self, convert: NumericConversion) -> Option<FloatType> {
		match self.value() {
			Type::Float(float) => Some(float.clone()),
			Type::Bool if convert >= NumericConversion::FromBool => Some(FloatType::F64),
			Type::String if convert >= NumericConversion::Parse => Some(FloatType::F64),
			_ => None,
		}
	}

	/// Convert the value to a boolean.
	pub fn to_bool(value: &Value) -> Result<bool> {
		let (_, bool) = Self::to_bool_output(value)?;
		Ok(bool)
	}

	/// Convert the value to the output of a short-circuit boolean operator.
	///
	/// This will preserve values that can be interpreted as bool so that
	/// they can be returned as the result of the operator.
	pub fn to_bool_output(value: &Value) -> Result<(Value, bool)> {
		let value = match value {
			Value::Unit => (Value::Unit, false),
			Value::Never => (Value::Never, false),
			Value::Null => (Value::Null, false),
			value @ Value::Bool(v) => (value.clone(), *v),
			value @ Value::Int(v) => (value.clone(), !v.is_zero()),
			value @ Value::Float(v) => (value.clone(), v.as_bool()),
			value @ Value::String(v) => (value.clone(), v.len() > 0),
		};
		Ok(value)
	}

	pub fn bool_output(&self) -> Option<Type> {
		match self {
			Type::Unknown => None,
			Type::Any => None,
			Type::Unit => Some(Type::Unit),
			Type::Null => Some(Type::Null),
			Type::Never => Some(Type::Never),
			Type::Bool => Some(Type::Bool),
			Type::String => Some(Type::String),
			Type::Int(value) => Some(Type::Int(value.clone())),
			Type::Float(value) => Some(Type::Float(value.clone())),
			Type::Or(a, b) => {
				let a = Self::bool_output(a);
				let b = Self::bool_output(b);
				if let Some(a) = a {
					if let Some(b) = b {
						Some(Type::Or(a.into(), b.into()))
					} else {
						None
					}
				} else {
					None
				}
			}
			typ @ Type::Ref(value) => {
				if Self::bool_output(value).is_some() {
					Some(typ.clone())
				} else {
					None
				}
			}
		}
	}
}

impl Display for Type {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.name())
	}
}
