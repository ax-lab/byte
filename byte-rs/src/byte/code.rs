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
pub mod op_and;
pub mod op_div;
pub mod op_minus;
pub mod op_mod;
pub mod op_mul;
pub mod op_neg;
pub mod op_not;
pub mod op_or;
pub mod op_plus;
pub mod op_sub;
pub mod runtime_scope;
pub mod values;

pub use op::*;
pub use op_add::*;
pub use op_and::*;
pub use op_div::*;
pub use op_minus::*;
pub use op_mod::*;
pub use op_mul::*;
pub use op_neg::*;
pub use op_not::*;
pub use op_or::*;
pub use op_plus::*;
pub use op_sub::*;
pub use runtime_scope::*;
pub use values::*;

use super::*;

const DEBUG_NODES: bool = false;

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
		let mut errors = Errors::new();
		for it in self.iter() {
			let node = self.generate_node(context, &it).handle(&mut errors);
			output.push(node);
			if errors.len() >= MAX_ERRORS {
				break;
			}
		}

		if errors.len() > 0 && DEBUG_NODES {
			println!("\n----- NODE DUMP -----\n");
			let mut output = String::new();
			let _ = self.output(ReprMode::Debug, ReprFormat::Full, &mut output);
			println!("{output}");
			println!("\n---------------------");
		}

		Ok(output).unless(errors)
	}

	fn generate_node(&self, context: &mut CodeContext, node: &NodeData) -> Result<Expr> {
		let compiler = context.compiler.get();
		let value = match node.get() {
			Node::Boolean(value) => {
				let value = ValueExpr::Bool(*value);
				Expr::Value(value)
			}
			Node::Integer(value) => {
				let value = IntValue::new(*value, DEFAULT_INT);
				Expr::Value(ValueExpr::Int(value))
			}
			Node::Null => {
				// TODO: figure out null
				Expr::Unit
			}
			Node::Literal(value) => {
				let value = StrValue::new(value, &compiler);
				Expr::Value(ValueExpr::Str(value))
			}
			Node::Line(list) => list.generate_expr(context)?,
			Node::Group(list) => list.generate_expr(context)?,
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
			Node::UnaryOp(op, arg) => {
				let arg = arg.generate_expr(context)?;
				let op = op.for_type(&arg.get_type())?;
				Expr::Unary(op, arg.into())
			}
			Node::BinaryOp(op, lhs, rhs) => {
				let lhs = lhs.generate_expr(context)?;
				let rhs = rhs.generate_expr(context)?;
				let op = op.for_types(&lhs.get_type(), &rhs.get_type())?;
				Expr::Binary(op, lhs.into(), rhs.into())
			}
			Node::Sequence(list) => {
				let mut errors = Errors::new();
				let mut sequence = Vec::new();
				for it in list.iter() {
					let it = it.generate_expr(context).handle(&mut errors);
					sequence.push(it);
					if errors.len() >= MAX_ERRORS {
						break;
					}
				}
				if !errors.empty() {
					return Err(errors);
				}
				Expr::Sequence(sequence)
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
			0 => Expr::Unit,
			_ => {
				let mut output = Vec::new();
				for it in self.iter() {
					let node = self.generate_node(context, &it)?;
					output.push(node);
				}
				Expr::Sequence(output)
			}
		};
		Ok(expr)
	}
}

//====================================================================================================================//
// Expressions
//====================================================================================================================//

/// Enumeration of builtin root expressions.
#[derive(Clone, Debug, Default)]
pub enum Expr {
	#[default]
	Never,
	Unit,
	Declare(Name, Option<usize>, Arc<Expr>),
	Value(ValueExpr),
	Variable(Name, Option<usize>, Type),
	Print(Arc<Expr>, &'static str),
	Unary(UnaryOpImpl, Arc<Expr>),
	Binary(BinaryOpImpl, Arc<Expr>, Arc<Expr>),
	Sequence(Vec<Expr>),
}

impl Expr {
	pub fn get_type(&self) -> Type {
		match self {
			Expr::Never => Type::Never,
			Expr::Unit => Type::Unit,
			Expr::Declare(.., expr) => expr.get_type(),
			Expr::Value(value) => Type::Value(value.get_type()),
			Expr::Variable(.., kind) => kind.clone(),
			Expr::Print(..) => Type::Unit,
			Expr::Unary(op, ..) => op.get().get_type(),
			Expr::Binary(op, ..) => op.get().get_type(),
			Expr::Sequence(list) => list.last().map(|x| x.get_type()).unwrap_or_else(|| Type::Unit),
		}
	}

	pub fn execute(&self, scope: &mut RuntimeScope) -> Result<Value> {
		match self {
			Expr::Never => {
				let error = format!("never expression cannot be evaluated");
				Err(Errors::from(error))
			}
			Expr::Unit => Ok(Value::from(())),
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
				let list = expr.as_sequence();
				let mut values = Vec::new();
				for expr in list {
					let value = expr.execute(scope)?;
					if !value.is_unit() {
						values.push(value)
					}
				}

				let mut output = scope.stdout();
				for (i, it) in values.into_iter().enumerate() {
					if i > 0 {
						write!(output, " ")?;
					}
					write!(output, "{it}")?;
				}
				write!(output, "{tail}")?;
				Ok(Value::from(()))
			}
			Expr::Unary(op, arg) => op.get().execute(scope, &arg),
			Expr::Binary(op, lhs, rhs) => op.get().execute(scope, lhs, rhs),
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

	pub fn as_sequence(&self) -> Vec<Expr> {
		let mut output = Vec::new();
		match self {
			Expr::Sequence(list) => output = list.clone(),
			expr => output.push(expr.clone()),
		}
		output
	}
}

//====================================================================================================================//
// Types
//====================================================================================================================//

/// Enumeration of builtin types.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Type {
	Unit,
	Never,
	Value(ValueType),
}

impl Type {
	pub fn validate_value(&self, value: &Value) -> Result<()> {
		let valid = match self {
			Type::Unit => value.is::<()>(),
			Type::Never => false,
			Type::Value(kind) => kind.is_valid_value(value),
		};
		if valid {
			Ok(())
		} else {
			let typ = value.type_name();
			Err(Errors::from(format!(
				"value `{value}` of type `{typ}` is not valid {self:?}"
			)))
		}
	}

	pub fn is_string(&self) -> bool {
		self == &Type::Value(ValueType::Str)
	}

	pub fn is_boolean(&self) -> bool {
		self == &Type::Value(ValueType::Bool)
	}

	pub fn is_int(&self) -> bool {
		self.get_int_type().is_some()
	}

	pub fn get_int_type(&self) -> Option<&IntType> {
		match self {
			Type::Value(ValueType::Int(int)) => Some(int),
			_ => None,
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
