use super::print;

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum Type {
	Unit,
	Never,
	Bool,
	Int(TypeInt),
	Float(TypeFloat),
}

#[derive(Copy, Clone, Eq, PartialEq)]
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

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum TypeFloat {
	Any,
	F32,
	F64,
}
