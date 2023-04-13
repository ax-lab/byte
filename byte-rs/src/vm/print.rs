use std::fmt::*;

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
			if val.typ() != &Type::Unit {
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

impl std::fmt::Display for Value {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		print::print_value(self.typ(), self.val(), f)
	}
}

impl std::fmt::Debug for Value {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "⸨")?;
		write!(f, "{:?}:=", self.typ())?;
		print::print_value(self.typ(), self.val(), f)?;
		write!(f, "⸩")
	}
}

impl std::fmt::Debug for Type {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		print::print_type(self, f)
	}
}

fn print_value(typ: &Type, val: &InnerValue, f: &mut Formatter) -> Result {
	match typ {
		Type::Never => write!(f, "!"),
		Type::Unit => write!(f, "()"),
		Type::Bool => write!(f, "{}", unsafe { val.bool }),
		Type::Int(typ) => {
			let val = unsafe { &val.int };
			match typ {
				TypeInt::Any => write!(f, "{}", unsafe { val.any }),
				TypeInt::I8 => write!(f, "{}", unsafe { val.i8 }),
				TypeInt::U8 => write!(f, "{}", unsafe { val.u8 }),
				TypeInt::I16 => write!(f, "{}", unsafe { val.i16 }),
				TypeInt::I32 => write!(f, "{}", unsafe { val.i32 }),
				TypeInt::I64 => write!(f, "{}", unsafe { val.i64 }),
				TypeInt::U16 => write!(f, "{}", unsafe { val.u16 }),
				TypeInt::U32 => write!(f, "{}", unsafe { val.u32 }),
				TypeInt::U64 => write!(f, "{}", unsafe { val.u64 }),
				TypeInt::ISize => write!(f, "{}", unsafe { val.isize }),
				TypeInt::USize => write!(f, "{}", unsafe { val.usize }),
			}
		}
		Type::Float(typ) => {
			let val = unsafe { &val.float };
			match typ {
				TypeFloat::Any => write!(f, "{}", unsafe { val.any }),
				TypeFloat::F32 => write!(f, "{}", unsafe { val.f32 }),
				TypeFloat::F64 => write!(f, "{}", unsafe { val.f64 }),
			}
		}
		Type::Other(typ) => typ.get().fmt_val(val, f),
	}
}

fn print_type(typ: &Type, f: &mut Formatter) -> Result {
	match typ {
		Type::Never => write!(f, "Never"),
		Type::Unit => write!(f, "Unit"),
		Type::Bool => write!(f, "Bool"),
		Type::Int(typ) => match typ {
			TypeInt::Any => write!(f, "Int⟨_⟩"),
			TypeInt::I8 => write!(f, "Int⟨8⟩"),
			TypeInt::U8 => write!(f, "Uint⟨8⟩"),
			TypeInt::I16 => write!(f, "Int⟨16⟩"),
			TypeInt::I32 => write!(f, "Int⟨32⟩"),
			TypeInt::I64 => write!(f, "Int⟨64⟩"),
			TypeInt::U16 => write!(f, "Uint⟨16⟩"),
			TypeInt::U32 => write!(f, "Uint⟨32⟩"),
			TypeInt::U64 => write!(f, "Uint⟨64⟩"),
			TypeInt::ISize => write!(f, "Int⟨size⟩"),
			TypeInt::USize => write!(f, "Uint⟨size⟩"),
		},
		Type::Float(typ) => match typ {
			TypeFloat::Any => write!(f, "Float⟨_⟩"),
			TypeFloat::F32 => write!(f, "Float⟨32⟩"),
			TypeFloat::F64 => write!(f, "Float⟨64⟩"),
		},
		Type::Other(typ) => {
			write!(f, "Type({})", typ.get())
		}
	}
}
