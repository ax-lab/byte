use super::*;

#[repr(C)]
pub union Val {
	pub(crate) int: IntVal,
}

impl Val {
	pub fn zero() -> Self {
		unsafe { std::mem::zeroed() }
	}

	pub fn int(&self) -> IntVal {
		unsafe { self.int }
	}
}
