#[repr(C)]
#[derive(Copy, Clone)]
pub union IntVal {
	u8: u8,
	i8: i8,
	u16: u16,
	i16: i16,
	u32: u32,
	i32: i32,
	u64: u64,
	i64: i64,
	usize: usize,
	isize: isize,
}

#[derive(Copy, Clone)]
pub enum IntType {
	U8,
	I8,
	U16,
	I16,
	U32,
	I32,
	U64,
	I64,
	USize,
	ISize,
}

impl std::fmt::Debug for IntType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::U8 => write!(f, "U8"),
			Self::I8 => write!(f, "I8"),
			Self::U16 => write!(f, "U16"),
			Self::I16 => write!(f, "I16"),
			Self::U32 => write!(f, "U32"),
			Self::I32 => write!(f, "I32"),
			Self::U64 => write!(f, "U64"),
			Self::I64 => write!(f, "I64"),
			Self::USize => write!(f, "USize"),
			Self::ISize => write!(f, "ISize"),
		}
	}
}

impl IntVal {
	pub fn i8(&self) -> i8 {
		unsafe { self.i8 }
	}

	pub fn u8(&self) -> u8 {
		unsafe { self.u8 }
	}

	pub fn i16(&self) -> i16 {
		unsafe { self.i16 }
	}

	pub fn u16(&self) -> u16 {
		unsafe { self.u16 }
	}

	pub fn i32(&self) -> i32 {
		unsafe { self.i32 }
	}

	pub fn u32(&self) -> u32 {
		unsafe { self.u32 }
	}

	pub fn i64(&self) -> i64 {
		unsafe { self.i64 }
	}

	pub fn u64(&self) -> u64 {
		unsafe { self.u64 }
	}

	pub fn isize(&self) -> isize {
		unsafe { self.isize }
	}

	pub fn usize(&self) -> usize {
		unsafe { self.usize }
	}
}
