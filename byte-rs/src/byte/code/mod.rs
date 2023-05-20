pub mod expr;
pub mod int;
pub mod typ;
pub mod val;

pub use expr::*;
pub use int::*;
pub use typ::*;
pub use val::*;

pub use expr::Expr;

pub struct Code {
	_list: Vec<Expr>,
}
