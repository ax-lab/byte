use super::*;

#[derive(Copy, Clone)]
pub struct Var(pub Type, pub Value);

#[repr(C)]
#[derive(Copy, Clone)]
pub union Value {
	pub bool: bool,
	pub int: ValueInt,
	pub float: ValueFloat,
	pub ptr: *const (),
}

#[derive(Copy, Clone)]
pub union ValueInt {
	pub any: usize,
	pub i8: i8,
	pub u8: u8,
	pub i16: i16,
	pub i32: i32,
	pub i64: i64,
	pub u16: u16,
	pub u32: u32,
	pub u64: u64,
	pub isize: isize,
	pub usize: usize,
}

#[derive(Copy, Clone)]
pub union ValueFloat {
	pub any: f64,
	pub f32: f32,
	pub f64: f64,
}

impl Var {
	fn typ(&self) -> &Type {
		&self.0
	}

	fn val(&self) -> &Value {
		&self.1
	}
}

impl std::fmt::Display for Var {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		print::print_value(self.typ(), self.val(), f)
	}
}

impl std::fmt::Debug for Var {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "⸨")?;
		write!(f, "{:?}:=", self.typ())?;
		print::print_value(self.typ(), self.val(), f)?;
		write!(f, "⸩")
	}
}
