use super::*;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum BinaryOp {
	Add,
	Mul,
}

impl BinaryOp {
	pub fn for_type(self, lhs: &Type) -> Result<BinaryOpImpl> {
		self.for_types(lhs, lhs)
	}

	pub fn for_types(&self, lhs: &Type, rhs: &Type) -> Result<BinaryOpImpl> {
		match self {
			BinaryOp::Add => match OpAdd::for_types(lhs, rhs) {
				Some(op) => Ok(BinaryOpImpl::from(op)),
				None => {
					let error = format!("operator `{self:?}` is not defined for `{lhs}` and `{rhs}`");
					let error = Errors::from(error);
					Err(error)
				}
			},
			BinaryOp::Mul => match OpMul::for_types(lhs, rhs) {
				Some(op) => Ok(BinaryOpImpl::from(op)),
				None => {
					let error = format!("operator `{self:?}` is not defined for `{lhs}` and `{rhs}`");
					let error = Errors::from(error);
					Err(error)
				}
			},
		}
	}
}

pub trait IsBinaryOp: IsValue + WithDebug {
	fn execute(&self, lhs: Value, rhs: Value) -> Result<Value>;
	fn get_type(&self) -> Type;
}

#[derive(Clone)]
pub struct BinaryOpImpl {
	inner: Arc<dyn IsBinaryOp>,
}

impl<T: IsBinaryOp> From<T> for BinaryOpImpl {
	fn from(value: T) -> Self {
		BinaryOpImpl { inner: Arc::new(value) }
	}
}

impl BinaryOpImpl {
	pub fn get(&self) -> &dyn IsBinaryOp {
		get_trait!(self, IsBinaryOp).unwrap()
	}
}

impl HasTraits for BinaryOpImpl {
	fn get_trait(&self, type_id: std::any::TypeId) -> Option<&dyn HasTraits> {
		with_trait!(self, type_id, WithDebug);
		self.inner.get_trait(type_id)
	}
}

impl Debug for BinaryOpImpl {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		self.get().fmt_debug(f)
	}
}
