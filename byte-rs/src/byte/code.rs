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
			Node::Line(list) => list.generate_expr(compiler)?,
			Node::Let(name, offset, list) => {
				let code = list.generate_code(compiler)?;
				Expr::Declare(name.clone(), Some(*offset), Arc::new(Expr::Sequence(code)))
			}
			Node::BinaryOp(op, lhs, rhs) => {
				let lhs = lhs.generate_expr(compiler)?;
				let rhs = rhs.generate_expr(compiler)?;
				let op = op.for_types(&lhs.get_type(), &rhs.get_type())?;
				Expr::Binary(op, lhs.into(), rhs.into())
			}
			value => {
				let mut error = format!("cannot generate code for `{value:?}`");
				{
					let mut output = error.indented();
					let _ = write!(output, "\n\n");
					let _ = self.output(ReprMode::Debug, ReprFormat::Full, &mut output);
				}
				let error = Errors::from_at(error, node.span().clone());
				return Err(error);
			}
		};
		Ok(value)
	}

	fn generate_expr(&self, compiler: &Compiler) -> Result<Expr> {
		let expr = match self.len() {
			0 => Expr::Value(ValueExpr::Unit),
			_ => {
				let code = self.generate_code(compiler)?;
				Expr::Sequence(code)
			}
		};
		Ok(expr)
	}
}

//====================================================================================================================//
// Expressions
//====================================================================================================================//

/// Enumeration of builtin root expressions.
#[derive(Clone, Debug)]
pub enum Expr {
	Declare(Name, Option<usize>, Arc<Expr>),
	Value(ValueExpr),
	Variable(Name, Option<usize>, Type),
	Binary(BinaryOpImpl, Arc<Expr>, Arc<Expr>),
	Sequence(Vec<Expr>),
}

impl Expr {
	pub fn get_type(&self) -> Type {
		match self {
			Expr::Declare(.., expr) => expr.get_type(),
			Expr::Value(value) => Type::Value(value.get_type()),
			Expr::Variable(.., kind) => kind.clone(),
			Expr::Binary(op, ..) => op.get().get_type(),
			Expr::Sequence(list) => list
				.last()
				.map(|x| x.get_type())
				.unwrap_or_else(|| Type::Value(ValueType::Unit)),
		}
	}

	pub fn execute(&self, scope: &mut RuntimeScope) -> Result<Value> {
		match self {
			Expr::Declare(name, offset, expr) => {
				let value = expr.execute(scope)?;
				scope.set(name.clone(), *offset, value.clone());
				Ok(value)
			}
			Expr::Value(value) => value.execute(scope),
			Expr::Variable(name, index, ..) => match scope.get(name, *index).cloned() {
				Some(value) => Ok(value),
				None => Err(Errors::from(format!("variable {name} not set"))),
			},
			Expr::Binary(op, lhs, rhs) => {
				let lhs = lhs.execute(scope)?;
				let rhs = rhs.execute(scope)?;
				op.get().execute(lhs, rhs)
			}
			Expr::Sequence(list) => {
				let mut value = Value::from(());
				for it in list.iter() {
					let next = it.execute(scope)?;
					value = Value::from(next);
				}
				Ok(value)
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

impl Display for Type {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "{self:?}")
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
		let a = Expr::Value(ValueExpr::Int(IntValue::new(2, IntType::I32)));
		let b = Expr::Value(ValueExpr::Int(IntValue::new(3, IntType::I32)));

		let op = BinaryOpImpl::from(OpAdd::for_type(&a.get_type()).unwrap());

		let expr = Expr::Binary(op, a.into(), b.into());

		let mut scope = RuntimeScope::new();
		let result = expr.execute(&mut scope)?;
		assert_eq!(result, Value::from(5));

		Ok(())
	}
}
