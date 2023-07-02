//! Expression for plain values, such as literal numbers and strings.

use super::*;

//====================================================================================================================//
// Builtin Literal Values
//====================================================================================================================//

#[derive(Clone, Debug)]
pub enum ValueExpr {
	Bool(bool),
	Str(StrValue),
	Int(IntValue),
	Float(FloatValue),
}

impl ValueExpr {
	pub fn get_type(&self) -> ValueType {
		match self {
			ValueExpr::Bool(..) => ValueType::Bool,
			ValueExpr::Str(..) => ValueType::Str,
			ValueExpr::Int(int) => ValueType::Int(int.get_type()),
			ValueExpr::Float(float) => ValueType::Float(float.get_type()),
		}
	}

	pub fn execute(&self, scope: &mut RuntimeScope) -> Result<Value> {
		match self {
			ValueExpr::Bool(value) => Ok(Value::from(*value)),
			ValueExpr::Str(value) => Ok(Value::from(value.get())),
			ValueExpr::Int(value) => value.execute(scope),
			ValueExpr::Float(_) => todo!(),
		}
	}
}

#[derive(Clone, Debug)]
pub struct StrValue(CompilerHandle<String>);

impl StrValue {
	pub fn new<T: AsRef<str>>(str: T, compiler: &Compiler) -> Self {
		let handle = compiler.store(str.as_ref().to_string());
		Self(handle)
	}

	pub fn get(&self) -> String {
		self.0.get().to_string()
	}
}

#[derive(Clone)]
pub struct IntValue {
	data: u128,
	base: u8,
	kind: IntType,
}

impl IntValue {
	pub fn new(value: u128, kind: IntType) -> Self {
		assert!(value <= kind.max_value());
		IntValue {
			data: value,
			kind,
			base: 10,
		}
	}

	pub fn value(&self) -> u128 {
		self.data
	}

	pub fn with_base(&self, base: u8) -> Self {
		let mut value = self.clone();
		value.base = base;
		value
	}

	pub fn get_type(&self) -> IntType {
		self.kind
	}

	pub fn execute(&self, scope: &mut RuntimeScope) -> Result<Value> {
		let _ = scope;
		let value = self.value();
		let value = match self.get_type() {
			IntType::I8 => Value::from(value as i8),
			IntType::U8 => Value::from(value as u8),
			IntType::I16 => Value::from(value as i16),
			IntType::U16 => Value::from(value as u16),
			IntType::I32 => Value::from(value as i32),
			IntType::U32 => Value::from(value as u32),
			IntType::I64 => Value::from(value as i64),
			IntType::U64 => Value::from(value as u64),
			IntType::I128 => Value::from(value as i128),
			IntType::U128 => Value::from(value as u128),
		};
		Ok(value)
	}
}

#[derive(Clone, Debug)]
pub enum FloatValue {
	NaN(FloatType),
	Infinity(FloatType),
	Value {
		kind: FloatType,
		base: u8,
		mantissa: CompilerHandle<Vec<u8>>,
		exp: i32,
	},
}

impl FloatValue {
	pub fn get_type(&self) -> FloatType {
		match self {
			FloatValue::NaN(kind) => *kind,
			FloatValue::Infinity(kind) => *kind,
			FloatValue::Value { kind, .. } => *kind,
		}
	}
}

//====================================================================================================================//
// Builtin Types
//====================================================================================================================//

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValueType {
	Bool,
	Str,
	Int(IntType),
	Float(FloatType),
}

impl ValueType {
	pub fn is_valid_value(&self, value: &Value) -> bool {
		match self {
			ValueType::Bool => value.is::<bool>(),
			ValueType::Str => value.is::<String>(),
			ValueType::Int(int) => int.is_valid_value(value),
			ValueType::Float(float) => float.is_valid_value(value),
		}
	}
}

pub const DEFAULT_INT: IntType = IntType::I64;

pub const fn int(value: i64) -> i64 {
	value
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IntType {
	I8,
	U8,
	I16,
	U16,
	I32,
	U32,
	I64,
	U64,
	I128,
	U128,
}

impl IntType {
	pub fn max_value(&self) -> u128 {
		match self {
			IntType::I8 => i8::MAX as u128,
			IntType::U8 => u8::MAX as u128,
			IntType::I16 => i16::MAX as u128,
			IntType::U16 => u16::MAX as u128,
			IntType::I32 => i32::MAX as u128,
			IntType::U32 => u32::MAX as u128,
			IntType::I64 => i64::MAX as u128,
			IntType::U64 => u64::MAX as u128,
			IntType::I128 => i128::MAX as u128,
			IntType::U128 => u128::MAX as u128,
		}
	}

	pub fn is_valid_value(&self, value: &Value) -> bool {
		match self {
			IntType::I8 => value.is::<i8>(),
			IntType::U8 => value.is::<u8>(),
			IntType::I16 => value.is::<i16>(),
			IntType::U16 => value.is::<u16>(),
			IntType::I32 => value.is::<i32>(),
			IntType::U32 => value.is::<u32>(),
			IntType::I64 => value.is::<i64>(),
			IntType::U64 => value.is::<u64>(),
			IntType::I128 => value.is::<i128>(),
			IntType::U128 => value.is::<u128>(),
		}
	}
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FloatType {
	Single,
	Double,
}

impl FloatType {
	pub fn is_valid_value(&self, value: &Value) -> bool {
		match self {
			FloatType::Single => value.is::<f32>(),
			FloatType::Double => value.is::<f64>(),
		}
	}
}

//====================================================================================================================//
// Debug
//====================================================================================================================//

impl Debug for IntValue {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		let kind = self.kind;
		let data = self.data;
		write!(f, "IntValue({kind:?}: {data})")
	}
}
