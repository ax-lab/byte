use super::*;

#[derive(Copy, Clone)]
pub enum Type {
	Unit,
	Never,
	Int(IntType),
}

impl Type {
	pub fn clone_val(&self, val: &Val) -> Val {
		match self {
			Type::Unit => Val::zero(),
			Type::Never => Val::zero(),
			Type::Int(..) => Val {
				int: unsafe { val.int.clone() },
			},
		}
	}

	pub fn print_val(&self, output: &mut dyn std::fmt::Write, val: &Val) -> std::fmt::Result {
		match self {
			Type::Unit => write!(output, "()")?,
			Type::Never => write!(output, "(!)")?,
			Type::Int(typ) => {
				let val = val.int();
				match typ {
					IntType::U8 => write!(output, "{}", val.u8())?,
					IntType::I8 => write!(output, "{}", val.i8())?,
					IntType::U16 => write!(output, "{}", val.u16())?,
					IntType::I16 => write!(output, "{}", val.i16())?,
					IntType::U32 => write!(output, "{}", val.u32())?,
					IntType::I32 => write!(output, "{}", val.i32())?,
					IntType::U64 => write!(output, "{}", val.u64())?,
					IntType::I64 => write!(output, "{}", val.i64())?,
					IntType::USize => write!(output, "{}", val.usize())?,
					IntType::ISize => write!(output, "{}", val.isize())?,
				}
			}
		}
		Ok(())
	}
}

impl std::fmt::Debug for Type {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Type::Unit => write!(f, "Unit"),
			Type::Never => write!(f, "Never"),
			Type::Int(typ) => write!(f, "Int({typ:?})"),
		}
	}
}
