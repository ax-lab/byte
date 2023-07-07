use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum NumericConversion {
	None,
	BoolToInt,
	Parse,
}

pub const ARITHMETIC_CONVERSION: NumericConversion = NumericConversion::None;

pub fn arithmetic_output(lhs: &Type, rhs: &Type) -> Option<Type> {
	if let Some(lt) = lhs.get_int_type(ARITHMETIC_CONVERSION) {
		if let Some(rt) = rhs.get_int_type(ARITHMETIC_CONVERSION) {
			let output = IntType::merge_for_upcast(lt, rt);
			let output = Type::Int(output);
			Some(output)
		} else {
			None
		}
	} else {
		None
	}
}

pub fn numeric_output(arg: &Type, convert: NumericConversion) -> Option<(Type, bool)> {
	if let Some(int) = arg.get_int_type(convert) {
		Some((arg.clone(), int.signed()))
	} else {
		None
	}
}

//====================================================================================================================//
// Value
//====================================================================================================================//

pub const DEFAULT_INT: IntType = IntType::I64;

pub type DefaultInt = i64;

pub const fn int(value: DefaultInt) -> DefaultInt {
	value
}

impl Default for IntValue {
	fn default() -> Self {
		IntValue::I64(0)
	}
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum IntValue {
	I8(i8),
	U8(u8),
	I16(i16),
	U16(u16),
	I32(i32),
	U32(u32),
	I64(i64),
	U64(u64),
	I128(i128),
	U128(u128),
}

impl IntValue {
	pub fn new(value: u128, kind: IntType) -> Result<IntValue> {
		if value > kind.max_value() {
			err!("value overflows an {kind} (value is {value})")
		} else {
			let value = match kind {
				IntType::I8 => IntValue::I8(value as i8),
				IntType::U8 => IntValue::U8(value as u8),
				IntType::I16 => IntValue::I16(value as i16),
				IntType::U16 => IntValue::U16(value as u16),
				IntType::I32 => IntValue::I32(value as i32),
				IntType::U32 => IntValue::U32(value as u32),
				IntType::I64 => IntValue::I64(value as i64),
				IntType::U64 => IntValue::U64(value as u64),
				IntType::I128 => IntValue::I128(value as i128),
				IntType::U128 => IntValue::U128(value as u128),
			};
			Ok(value)
		}
	}

	pub fn new_signed(value: i128, kind: IntType) -> Result<IntValue> {
		if value > 0 && value as u128 > kind.max_value() {
			err!("value overflows {kind} (value is {value})")
		} else if value < kind.min_value() {
			if kind.signed() {
				err!("value underflows {kind} (value is {value})")
			} else {
				err!("invalid negative value for {kind} (value is {value})")
			}
		} else {
			let value = match kind {
				IntType::I8 => IntValue::I8(value as i8),
				IntType::U8 => IntValue::U8(value as u8),
				IntType::I16 => IntValue::I16(value as i16),
				IntType::U16 => IntValue::U16(value as u16),
				IntType::I32 => IntValue::I32(value as i32),
				IntType::U32 => IntValue::U32(value as u32),
				IntType::I64 => IntValue::I64(value as i64),
				IntType::U64 => IntValue::U64(value as u64),
				IntType::I128 => IntValue::I128(value as i128),
				IntType::U128 => IntValue::U128(value as u128),
			};
			Ok(value)
		}
	}

	pub fn get_type(&self) -> IntType {
		match self {
			IntValue::I8(..) => IntType::I8,
			IntValue::U8(..) => IntType::U8,
			IntValue::I16(..) => IntType::I16,
			IntValue::U16(..) => IntType::U16,
			IntValue::I32(..) => IntType::I32,
			IntValue::U32(..) => IntType::U32,
			IntValue::I64(..) => IntType::I64,
			IntValue::U64(..) => IntType::U64,
			IntValue::I128(..) => IntType::I128,
			IntValue::U128(..) => IntType::U128,
		}
	}

	pub fn cast_to(&self, kind: &IntType) -> Result<IntValue> {
		if self.get_type().signed() {
			let value = self.signed();
			Self::new_signed(value, kind.clone())
		} else {
			let value = self.unsigned();
			Self::new(value, kind.clone())
		}
	}

	pub fn get_as<T: IsIntType>(&self) -> T::Type {
		T::from_int(self)
	}

	pub fn is_zero(&self) -> bool {
		match self {
			IntValue::I8(value) => *value == 0,
			IntValue::U8(value) => *value == 0,
			IntValue::I16(value) => *value == 0,
			IntValue::U16(value) => *value == 0,
			IntValue::I32(value) => *value == 0,
			IntValue::U32(value) => *value == 0,
			IntValue::I64(value) => *value == 0,
			IntValue::U64(value) => *value == 0,
			IntValue::I128(value) => *value == 0,
			IntValue::U128(value) => *value == 0,
		}
	}

	pub fn signed(&self) -> i128 {
		self.get_as::<i128>()
	}

	pub fn unsigned(&self) -> u128 {
		self.get_as::<u128>()
	}
}

impl Display for IntValue {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		if self.get_type().signed() {
			write!(f, "{}", self.signed())
		} else {
			write!(f, "{}", self.unsigned())
		}
	}
}

//====================================================================================================================//
// Type
//====================================================================================================================//

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
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
	pub fn name(&self) -> StrValue {
		match self {
			IntType::I8 => "I8".into(),
			IntType::U8 => "U8".into(),
			IntType::I16 => "I16".into(),
			IntType::U16 => "U16".into(),
			IntType::I32 => "I32".into(),
			IntType::U32 => "U32".into(),
			IntType::I64 => "I64".into(),
			IntType::U64 => "U64".into(),
			IntType::I128 => "I128".into(),
			IntType::U128 => "U128".into(),
		}
	}

	pub fn get_by_min_bits(bits: u8, signed: bool) -> IntType {
		if bits <= 8 {
			if signed {
				IntType::I8
			} else {
				IntType::U8
			}
		} else if bits <= 16 {
			if signed {
				IntType::I16
			} else {
				IntType::U16
			}
		} else if bits <= 32 {
			if signed {
				IntType::I32
			} else {
				IntType::U32
			}
		} else if bits <= 64 {
			if signed {
				IntType::I64
			} else {
				IntType::U64
			}
		} else {
			if signed {
				IntType::I128
			} else {
				IntType::U128
			}
		}
	}

	pub fn signed(&self) -> bool {
		match self {
			IntType::I8 | IntType::I16 | IntType::I32 | IntType::I64 | IntType::I128 => true,
			IntType::U8 | IntType::U16 | IntType::U32 | IntType::U64 | IntType::U128 => false,
		}
	}

	pub fn merge_for_upcast(a: Self, b: Self) -> Self {
		if a.signed() == b.signed() {
			// same sign, use the largest of the types
			if a.bits() > b.bits() {
				a
			} else {
				b
			}
		} else {
			// opposite signs, result in signed with the largest number of bits
			let bits = std::cmp::max(a.bits(), b.bits());
			Self::get_by_min_bits(bits, true)
		}
	}

	pub fn bits(&self) -> u8 {
		match self {
			IntType::I8 => 8,
			IntType::U8 => 8,
			IntType::I16 => 16,
			IntType::U16 => 16,
			IntType::I32 => 32,
			IntType::U32 => 32,
			IntType::I64 => 64,
			IntType::U64 => 64,
			IntType::I128 => 128,
			IntType::U128 => 128,
		}
	}

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

	pub fn min_value(&self) -> i128 {
		match self {
			IntType::I8 => i8::MIN as i128,
			IntType::U8 => u8::MIN as i128,
			IntType::I16 => i16::MIN as i128,
			IntType::U16 => u16::MIN as i128,
			IntType::I32 => i32::MIN as i128,
			IntType::U32 => u32::MIN as i128,
			IntType::I64 => i64::MIN as i128,
			IntType::U64 => u64::MIN as i128,
			IntType::I128 => i128::MIN as i128,
			IntType::U128 => u128::MIN as i128,
		}
	}
}

impl Display for IntType {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		write!(f, "{}", self.name())
	}
}

//====================================================================================================================//
// IsIntType
//====================================================================================================================//

impl<T: IsIntType> From<T> for IntValue {
	fn from(value: T) -> Self {
		value.to_int()
	}
}

pub trait IsIntType {
	type Type: Copy + Clone;

	fn get_int_type(&self) -> IntType;

	fn to_int(&self) -> IntValue;

	fn from_int(value: &IntValue) -> Self::Type;
}

int_type!(I8: i8);
int_type!(U8: u8);

int_type!(I16: i16);
int_type!(U16: u16);

int_type!(I32: i32);
int_type!(U32: u32);

int_type!(I64: i64);
int_type!(U64: u64);

int_type!(I128: i128);
int_type!(U128: u128);

//====================================================================================================================//
// Trait macro
//====================================================================================================================//

mod macros {
	#[macro_export]
	macro_rules! int_type {
		($name:ident : $int:ty) => {
			impl_int_type!($name: $int);
		};
	}

	#[macro_export]
	macro_rules! impl_int_type {
		($name:ident : $int:ty) => {
			impl IsIntType for $int {
				type Type = $int;

				fn get_int_type(&self) -> IntType {
					IntType::$name
				}

				fn to_int(&self) -> IntValue {
					IntValue::$name(*self)
				}

				fn from_int(value: &IntValue) -> Self::Type {
					match value {
						IntValue::I8(value) => *value as Self::Type,
						IntValue::U8(value) => *value as Self::Type,
						IntValue::I16(value) => *value as Self::Type,
						IntValue::U16(value) => *value as Self::Type,
						IntValue::I32(value) => *value as Self::Type,
						IntValue::U32(value) => *value as Self::Type,
						IntValue::I64(value) => *value as Self::Type,
						IntValue::U64(value) => *value as Self::Type,
						IntValue::I128(value) => *value as Self::Type,
						IntValue::U128(value) => *value as Self::Type,
					}
				}
			}
		};
	}

	pub use int_type;
}
