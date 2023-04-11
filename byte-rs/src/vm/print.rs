use std::fmt::*;

use super::*;

pub fn print_value(typ: &Type, val: &Value, f: &mut Formatter) -> Result {
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
	}
}

pub fn print_type(typ: &Type, f: &mut Formatter) -> Result {
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
	}
}
