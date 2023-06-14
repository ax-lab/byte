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

pub mod int;
pub mod op;
pub mod op_add;
pub mod values;

pub use op::*;
pub use op_add::*;
pub use values::*;

use super::*;

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
	Binary(BinaryOp, Handle<Expr>, Handle<Expr>),
}

impl Expr {
	pub fn get_type(&self) -> Type {
		match self {
			Expr::Value(value) => Type::Value(value.get_type()),
			Expr::Binary(op, ..) => op.get().get_type(),
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
