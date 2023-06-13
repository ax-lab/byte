//! High-level intermediate representation for runnable and compilable code
//! based on expression trees.
//!
//! Provides a strongly-typed static representation for code that is close
//! in level to a C-like language.
//!
//! The goal of this module is to provide a code representation that is high
//! level enough to easily build from the initial code parsing and semantical
//! analysis, while being low-level enough to be trivial to interpret, compile,
//! or transpile.
//!
//! This code representation is fully static and serializable, with all types
//! resolved, symbols statically bound, values stored as plain byte data, and
//! any sort of dynamic code expansion and generation (e.g. macros) completed.

pub mod values;

pub use values::*;

use super::*;
use crate::eval::*;

use std::{
	fmt::{Debug, Display},
	ops::Deref,
	sync::Arc,
};

pub trait Compilable {
	fn compile(&self, node: &Node, context: &Context, errors: &mut Errors) -> Option<Expr>;
}

impl Node {
	pub fn as_compilable(&self) -> Option<&dyn Compilable> {
		get_trait!(self, Compilable)
	}
}

//====================================================================================================================//
// Expressions
//====================================================================================================================//

/// Enumeration of builtin root expressions.
#[derive(Clone, Debug)]
pub enum Expr {
	Value(ValueExpr),
}

impl Expr {
	pub fn get_type(&self) -> Type {
		match self {
			Expr::Value(value) => Type::Value(value.get_type()),
		}
	}
}

//====================================================================================================================//
// Types
//====================================================================================================================//

/// Enumeration of builtin types.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Type {
	Value(ValueType),
}

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
