use super::*;

#[derive(Clone)]
pub struct Var<T: IsType> {
	name: String,
	typ: T,
}

impl<T: IsType> Var<T> {}

impl<T: IsType> IsExpr<T> for Var<T> {}

impl<T: IsType> HasTraits for Var<T> {}

impl<T: IsType> Debug for Var<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let name = &self.name;
		let typ = self.typ;
		write!(f, "({name}: {typ})")
	}
}
