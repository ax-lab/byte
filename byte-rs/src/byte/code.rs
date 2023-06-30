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
pub mod runtime_scope;
pub mod values;

pub use op::*;
pub use op_add::*;
pub use runtime_scope::*;
pub use values::*;

use super::*;

impl NodeList {
	pub fn generate_code(&self, compiler: &Compiler) -> Result<Vec<Expr>> {
		let mut output = Vec::new();
		for it in self.iter() {
			let node = self.generate_node(compiler, &it)?;
			output.push(node);
		}
		Ok(output)
	}

	fn generate_node(&self, compiler: &Compiler, node: &NodeData) -> Result<Expr> {
		let value = match node.get() {
			Node::Integer(value) => {
				let value = IntValue::new(*value, DEFAULT_INT);
				Expr::Value(ValueExpr::Int(value))
			}
			Node::Literal(value) => {
				let value = StrValue::new(value, compiler);
				Expr::Value(ValueExpr::Str(value))
			}
			value => {
				let mut error = format!("cannot generate code for `{value:?}`");
				let _ = write!(error.indented(), "\n\n{self:?}");
				let error = Errors::from_at(error, node.span().clone());
				return Err(error);
			}
		};
		Ok(value)
	}
}

//====================================================================================================================//
// Expressions
//====================================================================================================================//

/// Enumeration of builtin root expressions.
#[derive(Clone, Debug)]
pub enum Expr {
	Value(ValueExpr),
	Variable(Name, Type),
	Binary(BinaryOp, CompilerHandle<Expr>, CompilerHandle<Expr>),
}

impl Expr {
	pub fn get_type(&self) -> Type {
		match self {
			Expr::Value(value) => Type::Value(value.get_type()),
			Expr::Variable(.., kind) => kind.clone(),
			Expr::Binary(op, ..) => op.get().get_type(),
		}
	}

	pub fn execute(&self, scope: &mut RuntimeScope) -> Result<Value> {
		match self {
			Expr::Value(value) => value.execute(scope),
			Expr::Variable(name, ..) => scope.get(name).cloned(),
			Expr::Binary(op, lhs, rhs) => {
				let lhs = lhs.get().execute(scope)?;
				let rhs = rhs.get().execute(scope)?;
				op.get().execute(lhs, rhs)
			}
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

impl Type {
	pub fn validate_value(&self, value: &Value) -> Result<()> {
		match self {
			Type::Value(kind) => kind.validate_value(value),
		}
	}
}

//====================================================================================================================//
// Tests
//====================================================================================================================//

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn basic_eval() -> Result<()> {
		let compiler = Compiler::new();
		let a = Expr::Value(ValueExpr::Int(IntValue::new(2, IntType::I32)));
		let b = Expr::Value(ValueExpr::Int(IntValue::new(3, IntType::I32)));

		let op = BinaryOp::from(OpAdd::for_type(&a.get_type()).unwrap());

		let a = compiler.store(a);
		let b = compiler.store(b);
		let expr = Expr::Binary(op, a, b);

		let mut scope = RuntimeScope::new();
		let result = expr.execute(&mut scope)?;
		assert_eq!(result, Value::from(5));

		Ok(())
	}

	#[test]
	fn variables() -> Result<()> {
		let compiler = Compiler::new();

		let name = compiler.get_name("x");
		let kind = Type::Value(ValueType::Int(IntType::I32));
		let x = Expr::Variable(name.clone(), kind.clone());

		let mut scope = RuntimeScope::new();
		scope.declare(name.clone(), kind)?;
		scope.set(&name, Value::from(42))?;

		let result = x.execute(&mut scope)?;
		assert_eq!(result, Value::from(42));

		Ok(())
	}
}
