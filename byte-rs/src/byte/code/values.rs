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

#[derive(Clone, Debug)]
pub struct IntValue {
	pub data: u128,
	pub base: u8,
	pub kind: IntType,
}

impl IntValue {
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FloatType {
	Single,
	Double,
}
