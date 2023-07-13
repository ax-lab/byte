use super::*;

//====================================================================================================================//
// Value
//====================================================================================================================//

#[derive(Clone, Debug)]
pub enum FloatValue {
	F32(f32),
	F64(f64),
}

impl FloatValue {
	pub fn new(value: f64, kind: FloatType) -> FloatValue {
		let value = match kind {
			FloatType::F32 => FloatValue::F32(value as f32),
			FloatType::F64 => FloatValue::F64(value),
		};
		value
	}

	pub fn new32(value: f32) -> FloatValue {
		FloatValue::F32(value)
	}

	pub fn bits(&self) -> usize {
		match self {
			FloatValue::F32(_) => 32,
			FloatValue::F64(_) => 64,
		}
	}

	pub fn as_f64(&self) -> f64 {
		match self {
			FloatValue::F32(v) => *v as f64,
			FloatValue::F64(v) => *v,
		}
	}

	pub fn as_bool(&self) -> bool {
		let value = self.as_f64();
		value != 0.0 && !value.is_nan()
	}

	pub fn cmp_bit(&self) -> i8 {
		let v = self.as_f64();
		if v.is_nan() {
			2
		} else if v.is_infinite() {
			if v.is_sign_negative() {
				-1
			} else {
				1
			}
		} else {
			0
		}
	}

	pub fn get_type(&self) -> FloatType {
		match self {
			FloatValue::F32(..) => FloatType::F32,
			FloatValue::F64(..) => FloatType::F64,
		}
	}
}

impl PartialEq for FloatValue {
	fn eq(&self, other: &Self) -> bool {
		match self {
			FloatValue::F32(a) => match other {
				FloatValue::F32(b) => a == b,
				FloatValue::F64(_) => false,
			},
			FloatValue::F64(a) => match other {
				FloatValue::F32(_) => false,
				FloatValue::F64(b) => a == b,
			},
		}
	}
}

impl Eq for FloatValue {}

impl Ord for FloatValue {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		self.bits().cmp(&other.bits()).then_with(|| {
			if let Some(cmp) = self.as_f64().partial_cmp(&other.as_f64()) {
				cmp
			} else {
				self.cmp_bit().cmp(&other.cmp_bit())
			}
		})
	}
}

impl PartialOrd for FloatValue {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl Display for FloatValue {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		write!(f, "{}", self.as_f64())
	}
}

impl Hash for FloatValue {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.as_f64().to_bits().hash(state)
	}
}

//====================================================================================================================//
// Type
//====================================================================================================================//

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum FloatType {
	F32,
	F64,
}

impl FloatType {
	pub fn name(&self) -> StringValue {
		match self {
			FloatType::F32 => "F32".into(),
			FloatType::F64 => "F64".into(),
		}
	}

	pub fn merge_for_upcast(a: Self, b: Self) -> Self {
		if a == b {
			a
		} else {
			Self::F64
		}
	}
}

impl Display for FloatType {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		write!(f, "{}", self.name())
	}
}
