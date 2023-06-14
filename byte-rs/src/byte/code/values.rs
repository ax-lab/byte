//! Expression for plain values, such as literal numbers and strings.

use super::*;

//====================================================================================================================//
// Builtin Literal Values
//====================================================================================================================//

#[derive(Clone, Debug)]
pub enum ValueExpr {
	Unit,
	Never,
	Bool(bool),
	Str(StrValue),
	Int(IntValue),
	Float(FloatValue),
}

impl ValueExpr {
	pub fn get_type(&self) -> ValueType {
		match self {
			ValueExpr::Unit => ValueType::Unit,
			ValueExpr::Never => ValueType::Never,
			ValueExpr::Bool(..) => ValueType::Bool,
			ValueExpr::Str(..) => ValueType::Str,
			ValueExpr::Int(int) => ValueType::Int(int.get_type()),
			ValueExpr::Float(float) => ValueType::Float(float.get_type()),
		}
	}
}

#[derive(Clone, Debug)]
pub struct StrValue(Handle<String>);

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
			base: 10,
			kind: IntType::I64,
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
}

#[derive(Clone, Debug)]
pub enum FloatValue {
	NaN(FloatType),
	Infinity(FloatType),
	Value {
		kind: FloatType,
		base: u8,
		mantissa: Handle<Vec<u8>>,
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
	Unit,
	Never,
	Bool,
	Str,
	Int(IntType),
	Float(FloatType),
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
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FloatType {
	Single,
	Double,
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
