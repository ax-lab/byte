//! High-level intermediate representation for runnable and compilable code.
//!
//! This module provides a static and strongly-typed representation of code
//! that is still close to the source, but fully resolved.
//!
//! As such, this representation is still easy enough to construct, while it
//! can also be directly interpreted, compiled, or transpiled.

pub mod int;
pub mod vars;

pub use int::*;
pub use vars::*;

use super::*;
use crate::eval::*;

use std::{
	fmt::{Debug, Display},
	ops::Deref,
	sync::Arc,
};

pub trait Compilable {
	fn compile(&self, node: &Node, context: &Context, errors: &mut Errors) -> Option<Arc<dyn IsCode>>;
}

impl Node {
	pub fn as_compilable(&self) -> Option<&dyn Compilable> {
		get_trait!(self, Compilable)
	}
}

pub trait IsCode: WithEval {}

impl<T: WithEval> IsCode for T {}

//====================================================================================================================//
// Traits
//====================================================================================================================//

/// Trait for a generic expression. Expressions are the core of the code
/// representation.
pub trait IsExpr<T: IsType>: IsValue + Debug {}

/// Trait representing any code type.
pub trait IsType: Default + Copy + Clone + Display + Debug + Eq + PartialEq + IsValue {
	/// Data storage for this type.
	///
	/// For primitive value types, this is the value itself. Complex types
	/// require a handle to the static value data.
	type Data: Copy + Clone + Debug + IsValue;

	fn new_value(&self, scope: &mut Scope, data: &Self::Data) -> Result<Value> {
		let _ = scope;
		Err(Errors::from(format!(
			"type {self} does not implement data instantiation (data = {data:?})"
		)))
	}
}

/// Trait for unary operators.
pub trait IsUnaryOp<T: IsType>: IsValue + Debug {
	fn eval(&self, scope: &mut Scope, value: Value) -> Result<Value> {
		let _ = scope;
		let typ = T::default();
		Err(Errors::from(format!(
			"unary operator {self:?} for {typ} not implemented (value = {value})"
		)))
	}
}

/// Trait for binary operators.
pub trait IsBinaryOp<T: IsType>: IsValue + Debug {
	fn eval(&self, scope: &mut Scope, lhs: Value, rhs: Value) -> Result<Value> {
		let _ = scope;
		let typ = T::default();
		Err(Errors::from(format!(
			"binary operator {self:?} for {typ} not implemented (lhs = {lhs}, rhs = {rhs})"
		)))
	}
}

//====================================================================================================================//
// Expr
//====================================================================================================================//

/// Basic expression types.
#[derive(Clone, Debug)]
pub enum Expr<T: IsType> {
	Value(T::Data),
	Unary(UnaryOp<T>, OpValue<T>),
	Binary(BinaryOp<T>, OpValue<T>, OpValue<T>),
}

impl<T: IsType> HasTraits for Expr<T> {
	fn get_trait(&self, type_id: std::any::TypeId) -> Option<&dyn HasTraits> {
		with_trait!(self, type_id, WithEval);
		None
	}
}

impl<T: IsType> IsExpr<T> for Expr<T> {}

impl<T: IsType> WithEval for Expr<T> {
	fn eval(&self, scope: &mut Scope) -> Result<Value> {
		match self {
			Expr::Value(data) => {
				let typ = T::default();
				typ.new_value(scope, data)
			}

			Expr::Unary(op, val) => {
				let val = scope.eval(val.as_ref())?;
				op.eval(scope, val)
			}
			Expr::Binary(op, lhs, rhs) => {
				let lhs = scope.eval(lhs.as_ref())?;
				let rhs = scope.eval(rhs.as_ref())?;
				op.eval(scope, lhs, rhs)
			}
		}
	}
}

//====================================================================================================================//
// Helper types
//====================================================================================================================//

/// Wraps a generic expression value.
#[derive(Clone, Debug)]
pub struct OpValue<T: IsType>(Arc<dyn IsExpr<T>>);

impl<T: IsType> OpValue<T> {
	pub fn new<U: IsExpr<T>>(value: U) -> Self {
		let value = Arc::new(value);
		Self(value)
	}
}

impl<T: IsType> OpValue<T> {
	pub fn as_ref(&self) -> &dyn IsExpr<T> {
		self.0.as_ref()
	}
}

impl<T: IsType> Deref for OpValue<T> {
	type Target = dyn IsExpr<T>;

	fn deref(&self) -> &Self::Target {
		self.as_ref()
	}
}

/// Wraps a generic unary operator.
#[derive(Clone, Debug)]
pub struct UnaryOp<T: IsType>(Arc<dyn IsUnaryOp<T>>);

impl<T: IsType> UnaryOp<T> {
	pub fn new<U: IsUnaryOp<T>>(value: U) -> Self {
		let value = Arc::new(value);
		Self(value)
	}
}

impl<T: IsType> UnaryOp<T> {
	pub fn as_ref(&self) -> &dyn IsUnaryOp<T> {
		self.0.as_ref()
	}
}

impl<T: IsType> Deref for UnaryOp<T> {
	type Target = dyn IsUnaryOp<T>;

	fn deref(&self) -> &Self::Target {
		self.as_ref()
	}
}

/// Wraps a generic binary operator.
#[derive(Clone, Debug)]
pub struct BinaryOp<T: IsType>(Arc<dyn IsBinaryOp<T>>);

impl<T: IsType> BinaryOp<T> {
	pub fn new<U: IsBinaryOp<T>>(value: U) -> Self {
		let value = Arc::new(value);
		Self(value)
	}
}

impl<T: IsType> BinaryOp<T> {
	pub fn as_ref(&self) -> &dyn IsBinaryOp<T> {
		self.0.as_ref()
	}
}

impl<T: IsType> Deref for BinaryOp<T> {
	type Target = dyn IsBinaryOp<T>;

	fn deref(&self) -> &Self::Target {
		self.as_ref()
	}
}
