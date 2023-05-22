pub mod expr;
pub mod int;
pub mod typ;
pub mod val;

use std::sync::Arc;

pub use expr::*;
pub use int::*;
pub use typ::*;
pub use val::*;

pub use expr::Expr;

#[derive(Clone, Default)]
pub struct Code {
	list: Arc<Vec<Expr>>,
}

impl Code {
	pub fn append(&mut self, expr: Expr) {
		let list = Arc::make_mut(&mut self.list);
		list.push(expr);
	}
}
