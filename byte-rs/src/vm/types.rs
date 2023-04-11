use super::print;

#[derive(Copy, Clone)]
pub enum Type {
	Unit,
	Never,
	Bool,
	Int(TypeInt),
	Float(TypeFloat),
}

#[derive(Copy, Clone)]
pub enum TypeInt {
	Any,
	I8,
	U8,
	I16,
	I32,
	I64,
	U16,
	U32,
	U64,
	ISize,
	USize,
}

#[derive(Copy, Clone)]
pub enum TypeFloat {
	Any,
	F32,
	F64,
}

impl std::fmt::Debug for Type {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		print::print_type(self, f)
	}
}
