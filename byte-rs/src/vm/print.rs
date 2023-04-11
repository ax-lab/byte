use super::*;

pub fn print_value(typ: &Type, val: &Value) {
	match typ {
		Type::Never => print!("!"),
		Type::Unit => print!("()"),
		Type::Bool => print!("{}", unsafe { val.bool }),
		Type::Int(typ) => {
			let val = unsafe { &val.int };
			match typ {
				TypeInt::Any => print!("{}", unsafe { val.any }),
				TypeInt::I8 => print!("{}", unsafe { val.i8 }),
				TypeInt::U8 => print!("{}", unsafe { val.u8 }),
				TypeInt::I16 => print!("{}", unsafe { val.i16 }),
				TypeInt::I32 => print!("{}", unsafe { val.i32 }),
				TypeInt::I64 => print!("{}", unsafe { val.i64 }),
				TypeInt::U16 => print!("{}", unsafe { val.u16 }),
				TypeInt::U32 => print!("{}", unsafe { val.u32 }),
				TypeInt::U64 => print!("{}", unsafe { val.u64 }),
				TypeInt::ISize => print!("{}", unsafe { val.isize }),
				TypeInt::USize => print!("{}", unsafe { val.usize }),
			}
		}
		Type::Float(typ) => {
			let val = unsafe { &val.float };
			match typ {
				TypeFloat::Any => print!("{}", unsafe { val.any }),
				TypeFloat::F32 => print!("{}", unsafe { val.f32 }),
				TypeFloat::F64 => print!("{}", unsafe { val.f64 }),
			}
		}
	}
}

pub fn print_type(typ: &Type) {
	match typ {
		Type::Never => print!("Never"),
		Type::Unit => print!("Unit"),
		Type::Bool => print!("Bool"),
		Type::Int(typ) => match typ {
			TypeInt::Any => print!("Int⟨_⟩"),
			TypeInt::I8 => print!("Int⟨8⟩"),
			TypeInt::U8 => print!("Uint⟨8⟩"),
			TypeInt::I16 => print!("Int⟨16⟩"),
			TypeInt::I32 => print!("Int⟨32⟩"),
			TypeInt::I64 => print!("Int⟨64⟩"),
			TypeInt::U16 => print!("Uint⟨16⟩"),
			TypeInt::U32 => print!("Uint⟨32⟩"),
			TypeInt::U64 => print!("Uint⟨64⟩"),
			TypeInt::ISize => print!("Int⟨size⟩"),
			TypeInt::USize => print!("Uint⟨size⟩"),
		},
		Type::Float(typ) => match typ {
			TypeFloat::Any => print!("Float⟨_⟩"),
			TypeFloat::F32 => print!("Float⟨32⟩"),
			TypeFloat::F64 => print!("Float⟨64⟩"),
		},
	}
}
