use super::*;

#[allow(unused)]
#[derive(Debug)]
pub enum ExprValue {
	Bool { value: bool },
	Integer { value: u64 },
	Literal { value: String },
}

impl IsNode for ExprValue {}
