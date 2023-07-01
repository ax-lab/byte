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
pub mod op_mul;
pub mod runtime_scope;
pub mod values;

pub use op::*;
pub use op_add::*;
pub use op_mul::*;
pub use runtime_scope::*;
pub use values::*;

use super::*;

pub struct CodeContext {
	compiler: CompilerRef,
	declares: HashMap<(Name, Option<usize>), Type>,
}

impl CodeContext {
	pub fn new(compiler: CompilerRef) -> Self {
		let compiler = compiler;
		Self {
			compiler,
			declares: Default::default(),
		}
	}
}

impl NodeList {
	pub fn generate_code(&self, context: &mut CodeContext) -> Result<Vec<Expr>> {
		let mut output = Vec::new();
		for it in self.iter() {
			let node = self.generate_node(context, &it)?;
			output.push(node);
		}
		Ok(output)
	}

	fn generate_node(&self, context: &mut CodeContext, node: &NodeData) -> Result<Expr> {
		let compiler = context.compiler.get();
		let value = match node.get() {
			Node::Integer(value) => {
				let value = IntValue::new(*value, DEFAULT_INT);
				Expr::Value(ValueExpr::Int(value))
			}
			Node::Literal(value) => {
				let value = StrValue::new(value, &compiler);
				Expr::Value(ValueExpr::Str(value))
			}
			Node::Line(list) => list.generate_expr(context)?,
			Node::Let(name, offset, list) => {
				let expr = list.generate_expr(context)?;
				let kind = expr.get_type();
				context.declares.insert((name.clone(), Some(*offset)), kind);
				Expr::Declare(name.clone(), Some(*offset), Arc::new(expr))
			}
			Node::Variable(name, index) => {
				if let Some(kind) = context.declares.get(&(name.clone(), *index)) {
					Expr::Variable(name.clone(), *index, kind.clone())
				} else {
					let error = format!("variable `{name}` ({index:?}) does not match any declaration");
					let error = Errors::from_at(error, node.span().clone());
					return Err(error);
				}
			}
			Node::Print(expr, tail) => {
				let expr = expr.generate_expr(context)?;
				Expr::Print(expr.into(), tail)
			}
			Node::BinaryOp(op, lhs, rhs) => {
				let lhs = lhs.generate_expr(context)?;
				let rhs = rhs.generate_expr(context)?;
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

	fn generate_expr(&self, context: &mut CodeContext) -> Result<Expr> {
		let expr = match self.len() {
			0 => Expr::Value(ValueExpr::Unit),
			_ => {
				let code = self.generate_code(context)?;
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
	Print(Arc<Expr>, &'static str),
	Binary(BinaryOpImpl, Arc<Expr>, Arc<Expr>),
	Sequence(Vec<Expr>),
}

impl Expr {
	pub fn get_type(&self) -> Type {
		match self {
			Expr::Declare(.., expr) => expr.get_type(),
			Expr::Value(value) => Type::Value(value.get_type()),
			Expr::Variable(.., kind) => kind.clone(),
			Expr::Print(..) => Type::Value(ValueType::Unit),
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
			Expr::Print(expr, tail) => {
				let value = expr.execute(scope)?;
				let mut output = scope.stdout();
				write!(output, "{value}{tail}")?;
				Ok(Value::from(()))
			}
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

	pub fn is_string(&self) -> bool {
		self == &Type::Value(ValueType::Str)
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
