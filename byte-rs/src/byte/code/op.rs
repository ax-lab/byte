use super::*;

//====================================================================================================================//
// UnaryOp
//====================================================================================================================//

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum UnaryOp {
	Not,
	Neg,
	Plus,
	Minus,
}

impl UnaryOp {
	pub fn for_type(&self, arg: &Type) -> Result<UnaryOpImpl> {
		let arg = arg.value();
		let result = match self {
			UnaryOp::Not => OpNot::for_type(arg).map(|op| UnaryOpImpl::from(op)),
			UnaryOp::Neg => OpNeg::for_type(arg).map(|op| UnaryOpImpl::from(op)),
			UnaryOp::Plus => OpPlus::for_type(arg).map(|op| UnaryOpImpl::from(op)),
			UnaryOp::Minus => OpMinus::for_type(arg).map(|op| UnaryOpImpl::from(op)),
		};

		match result {
			Some(op) => Ok(op),
			None => {
				let error = format!("operator `{self:?}` is not defined for `{arg}`");
				let error = Errors::from(error);
				Err(error)
			}
		}
	}
}

pub trait IsUnaryOp: IsValue + WithDebug {
	fn execute(&self, scope: &mut RuntimeScope, arg: &Expr) -> Result<ExprValue>;
	fn get_type(&self) -> Type;
}

#[derive(Clone)]
pub struct UnaryOpImpl {
	inner: Arc<dyn IsUnaryOp>,
}

impl<T: IsUnaryOp> From<T> for UnaryOpImpl {
	fn from(value: T) -> Self {
		UnaryOpImpl { inner: Arc::new(value) }
	}
}

impl UnaryOpImpl {
	pub fn get(&self) -> &dyn IsUnaryOp {
		get_trait!(self, IsUnaryOp).unwrap()
	}
}

impl HasTraits for UnaryOpImpl {
	fn get_trait(&self, type_id: std::any::TypeId) -> Option<&dyn HasTraits> {
		with_trait!(self, type_id, WithDebug);
		self.inner.get_trait(type_id)
	}
}

impl Debug for UnaryOpImpl {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		self.get().fmt_debug(f)
	}
}

//====================================================================================================================//
// BinaryOp
//====================================================================================================================//

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum BinaryOp {
	Add,
	Sub,
	Mul,
	Div,
	Mod,
	And,
	Or,
	Assign,
}

impl BinaryOp {
	pub fn for_type(self, lhs: &Type) -> Result<BinaryOpImpl> {
		self.for_types(lhs, lhs)
	}

	pub fn for_types(&self, lhs: &Type, rhs: &Type) -> Result<BinaryOpImpl> {
		// TODO: find a solution to get a span here and/or to apply spans to chained errors.

		let lhs_ref = lhs;
		let lhs = lhs.value();
		let rhs = rhs.value();
		let result = match self {
			BinaryOp::Add => OpAdd::for_types(lhs, rhs).map(|op| BinaryOpImpl::from(op)),
			BinaryOp::Sub => OpSub::for_types(lhs, rhs).map(|op| BinaryOpImpl::from(op)),
			BinaryOp::Mul => OpMul::for_types(lhs, rhs).map(|op| BinaryOpImpl::from(op)),
			BinaryOp::Div => OpDiv::for_types(lhs, rhs).map(|op| BinaryOpImpl::from(op)),
			BinaryOp::Mod => OpMod::for_types(lhs, rhs).map(|op| BinaryOpImpl::from(op)),
			BinaryOp::And => OpAnd::for_types(lhs, rhs).map(|op| BinaryOpImpl::from(op)),
			BinaryOp::Or => OpOr::for_types(lhs, rhs).map(|op| BinaryOpImpl::from(op)),
			BinaryOp::Assign => {
				if lhs != rhs {
					let error = format!("cannot assign `{rhs:?}` to `{lhs:?}`");
					let error = Errors::from(error);
					return Err(error);
				} else if !matches!(lhs_ref, Type::Ref(..)) {
					let error = format!("cannot assign to non-reference `{lhs_ref}`");
					let error = Errors::from(error);
					return Err(error);
				}

				// TODO: the operator actually needs access to the whole expression
				Some(BinaryOpImpl::from(OpAssign(lhs.clone())))
			}
		};

		match result {
			Some(op) => Ok(op),
			None => {
				let error = format!("operator `{self:?}` is not defined for `{lhs}` and `{rhs}`");
				let error = Errors::from(error);
				Err(error)
			}
		}
	}
}

pub trait IsBinaryOp: IsValue + WithDebug {
	fn execute(&self, scope: &mut RuntimeScope, lhs: &Expr, rhs: &Expr) -> Result<ExprValue>;
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
