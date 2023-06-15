use super::*;

pub trait IsBinaryOp: IsValue + WithDebug {
	fn execute(&self, lhs: Value, rhs: Value) -> Result<Value>;
	fn get_type(&self) -> Type;
}

#[derive(Clone)]
pub struct BinaryOp {
	inner: Arc<dyn IsBinaryOp>,
}

impl<T: IsBinaryOp> From<T> for BinaryOp {
	fn from(value: T) -> Self {
		BinaryOp { inner: Arc::new(value) }
	}
}

impl BinaryOp {
	pub fn get(&self) -> &dyn IsBinaryOp {
		get_trait!(self, IsBinaryOp).unwrap()
	}
}

impl HasTraits for BinaryOp {
	fn get_trait(&self, type_id: std::any::TypeId) -> Option<&dyn HasTraits> {
		with_trait!(self, type_id, WithDebug);
		self.inner.get_trait(type_id)
	}
}

impl Debug for BinaryOp {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		self.get().fmt_debug(f)
	}
}
