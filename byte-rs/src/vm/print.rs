use std::fmt::*;

use crate::core::num::*;
use crate::core::*;

use super::expr::*;
use super::*;

#[derive(Debug)]
pub struct Print {
	pub args: Vec<Expr>,
	pub line: bool,
}

impl IsExpr for Print {
	fn eval(&self, rt: &mut Runtime) -> Value {
		let mut empty = true;
		for expr in self.args.iter() {
			if !empty {
				print!(" ");
			}
			let val = expr.val().eval(rt);
			if !val.is_unit() {
				empty = false;
				print!("{val}");
			}
		}
		if self.line {
			println!();
		}
		Value::unit()
	}

	fn get_type(&self) -> Type {
		Type::Unit
	}
}

impl std::fmt::Debug for Type {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		print::print_type(self, f)
	}
}

fn print_type(typ: &Type, f: &mut Formatter) -> Result {
	match typ {
		Type::Never => write!(f, "Never"),
		Type::Unit => write!(f, "Unit"),
		Type::Bool => write!(f, "Bool"),
		Type::String => write!(f, "String"),
		Type::Int(typ) => match typ {
			kind::Int::Any => write!(f, "Int⟨_⟩"),
			kind::Int::I8 => write!(f, "Int⟨8⟩"),
			kind::Int::U8 => write!(f, "Uint⟨8⟩"),
			kind::Int::I16 => write!(f, "Int⟨16⟩"),
			kind::Int::I32 => write!(f, "Int⟨32⟩"),
			kind::Int::I64 => write!(f, "Int⟨64⟩"),
			kind::Int::U16 => write!(f, "Uint⟨16⟩"),
			kind::Int::U32 => write!(f, "Uint⟨32⟩"),
			kind::Int::U64 => write!(f, "Uint⟨64⟩"),
			kind::Int::ISize => write!(f, "Int⟨size⟩"),
			kind::Int::USize => write!(f, "Uint⟨size⟩"),
		},
		Type::Float(typ) => match typ {
			kind::Float::Any => write!(f, "Float⟨_⟩"),
			kind::Float::F32 => write!(f, "Float⟨32⟩"),
			kind::Float::F64 => write!(f, "Float⟨64⟩"),
		},
		Type::Other(typ) => {
			write!(f, "Type({:?})", typ)
		}
	}
}
