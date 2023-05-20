use super::*;

#[derive(Clone)]
pub enum Expr {
	Unit,
	Never,
	Int(IntType, IntVal),
}

impl Expr {
	pub fn get_type(&self) -> Type {
		match self {
			Expr::Unit => Type::Unit,
			Expr::Never => Type::Never,
			Expr::Int(typ, ..) => Type::Int(*typ),
		}
	}
}
