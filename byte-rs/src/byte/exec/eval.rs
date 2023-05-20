use super::*;

impl Expr {
	pub fn eval(&self) -> Result<Value> {
		match self {
			Expr::Unit => Ok(Value::from(())),
			Expr::Never => Err("expression with Never type must never execute".into()),
			Expr::Int(typ, val) => Ok(match typ {
				IntType::U8 => Value::from(val.u8()),
				IntType::I8 => Value::from(val.i8()),
				IntType::U16 => Value::from(val.u16()),
				IntType::I16 => Value::from(val.i16()),
				IntType::U32 => Value::from(val.u32()),
				IntType::I32 => Value::from(val.i32()),
				IntType::U64 => Value::from(val.u64()),
				IntType::I64 => Value::from(val.i64()),
				IntType::USize => Value::from(val.usize()),
				IntType::ISize => Value::from(val.isize()),
			}),
		}
	}
}
