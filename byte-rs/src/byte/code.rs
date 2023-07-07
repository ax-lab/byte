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
pub mod op_assign;
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
pub use op_assign::*;
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

// TODO: figure out NULL
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Null;

has_traits!(Null: IsValue, WithDebug, WithDisplay);

impl Debug for Null {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "null")
	}
}

impl Display for Null {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "null")
	}
}

pub struct CodeContext {
	declares: HashMap<(Symbol, Option<usize>), Type>,
}

impl CodeContext {
	pub fn new() -> Self {
		Self {
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

		if (errors.len() > 0 && DEBUG_NODES) || DUMP_CODE {
			println!("\n----- NODE DUMP -----\n");
			let mut output = String::new();
			let _ = write!(output, "{self:?}");
			println!("{output}");
			println!("\n---------------------");
		}

		Ok(output).unless(errors)
	}

	fn generate_node(&self, context: &mut CodeContext, node: &Node) -> Result<Expr> {
		let value = match node.bit() {
			Bit::Boolean(value) => {
				let value = ValueExpr::Bool(*value);
				Expr::Value(value)
			}
			Bit::Integer(value) => {
				let value = IntValue::new(*value, DEFAULT_INT);
				Expr::Value(ValueExpr::Int(value))
			}
			Bit::Null => Expr::Null,
			Bit::Literal(value) => {
				let value = StrValue::new(value);
				Expr::Value(ValueExpr::Str(value))
			}
			Bit::Line(list) => list.generate_expr(context)?,
			Bit::Group(list) => list.generate_expr(context)?,
			Bit::Let(name, offset, list) => {
				let expr = list.generate_expr(context)?;
				let kind = expr.get_type();
				context.declares.insert((name.clone(), Some(*offset)), kind);
				Expr::Declare(name.clone(), Some(*offset), Arc::new(expr))
			}
			Bit::Variable(name, index) => {
				if let Some(kind) = context.declares.get(&(name.clone(), *index)) {
					Expr::Variable(name.clone(), *index, kind.clone())
				} else {
					let error = format!("variable `{name}` ({index:?}) does not match any declaration");
					let error = Errors::from(error, node.span().clone());
					return Err(error);
				}
			}
			Bit::Print(expr, tail) => {
				let expr = expr.generate_expr(context)?;
				Expr::Print(expr.into(), tail)
			}
			Bit::UnaryOp(op, arg) => {
				let arg = arg.generate_expr(context)?;
				let op = op.for_type(&arg.get_type())?;
				Expr::Unary(op, arg.into())
			}
			Bit::BinaryOp(op, lhs, rhs) => {
				let lhs = lhs.generate_expr(context)?;
				let rhs = rhs.generate_expr(context)?;
				let op = op.for_types(&lhs.get_type(), &rhs.get_type())?;
				Expr::Binary(op, lhs.into(), rhs.into())
			}
			Bit::Sequence(list) => {
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
			Bit::Conditional(a, b, c) => {
				let a = a.generate_expr(context)?;
				let b = b.generate_expr(context)?;
				let c = c.generate_expr(context)?;
				Expr::Conditional(a.into(), b.into(), c.into())
			}
			value => {
				let mut error = format!("cannot generate code for `{value:?}`");
				{
					let mut output = error.indented();
					let _ = write!(output, "\n\n");
					let _ = write!(output, "{self:?}");
				}
				let error = Errors::from(error, node.span().clone());
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
				if output.len() == 1 {
					output.into_iter().next().unwrap()
				} else {
					Expr::Sequence(output)
				}
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
	Null,
	Declare(Symbol, Option<usize>, Arc<Expr>),
	Conditional(Arc<Expr>, Arc<Expr>, Arc<Expr>),
	Value(ValueExpr),
	Variable(Symbol, Option<usize>, Type),
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
			Expr::Null => Type::Null,
			Expr::Declare(.., expr) => expr.get_type(),
			Expr::Value(value) => Type::Value(value.get_type()),
			Expr::Variable(.., kind) => Type::Ref(kind.clone().into()),
			Expr::Print(..) => Type::Unit,
			Expr::Unary(op, ..) => op.get().get_type(),
			Expr::Binary(op, ..) => op.get().get_type(),
			Expr::Sequence(list) => list.last().map(|x| x.get_type()).unwrap_or_else(|| Type::Unit),
			Expr::Conditional(_, a, b) => {
				let a = a.get_type();
				let b = b.get_type();
				if a == b {
					a
				} else {
					Type::Or(a.into(), b.into())
				}
			}
		}
	}

	pub fn execute(&self, scope: &mut RuntimeScope) -> Result<ExprValue> {
		match self {
			Expr::Never => {
				let error = format!("never expression cannot be evaluated");
				Err(Errors::from(error, Span::default()))
			}
			Expr::Unit => Ok(Value::from(()).into()),
			Expr::Null => Ok(Value::from(Null).into()),
			Expr::Declare(name, offset, expr) => {
				let value = expr.execute(scope)?;
				scope.set(name.clone(), *offset, value.clone().into());
				Ok(value)
			}
			Expr::Value(value) => value.execute(scope).map(|x| x.into()),
			Expr::Variable(name, index, ..) => match scope.get(name, *index).cloned() {
				Some(value) => Ok(ExprValue::Variable(name.clone(), index.clone(), value)),
				None => Err(Errors::from(format!("variable {name} not set"), Span::default())),
			},
			Expr::Print(expr, tail) => {
				let list = expr.as_sequence();
				let mut values = Vec::new();
				for expr in list.iter() {
					let value = expr.execute(scope)?.value();
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
				Ok(Value::from(()).into())
			}
			Expr::Unary(op, arg) => op.get().execute(scope, &arg),
			Expr::Binary(op, lhs, rhs) => op.get().execute(scope, lhs, rhs),
			Expr::Sequence(list) => {
				let mut value = Value::from(()).into();
				for it in list.iter() {
					value = it.execute(scope)?;
				}
				Ok(value)
			}
			Expr::Conditional(cond, a, b) => {
				let cond = cond.execute(scope)?;
				let cond = cond.value().to_bool()?;
				if cond {
					a.execute(scope)
				} else {
					b.execute(scope)
				}
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

#[derive(Clone, Debug)]
pub enum ExprValue {
	Value(Value),
	Variable(Symbol, Option<usize>, Value),
}

impl ExprValue {
	pub fn value(self) -> Value {
		match self {
			ExprValue::Value(value) => value,
			ExprValue::Variable(.., value) => value,
		}
	}
}

impl From<ExprValue> for Value {
	fn from(expr_value: ExprValue) -> Self {
		expr_value.value()
	}
}

impl From<Value> for ExprValue {
	fn from(value: Value) -> Self {
		ExprValue::Value(value)
	}
}

//====================================================================================================================//
// Types
//====================================================================================================================//

/// Enumeration of builtin types.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Type {
	Unit,
	Null,
	Never,
	Value(ValueType),
	Or(Arc<Type>, Arc<Type>),
	Ref(Arc<Type>),
}

impl Type {
	pub fn validate_value(&self, value: &Value) -> Result<()> {
		let valid = match self {
			Type::Unit => value.is::<()>(),
			Type::Never => false,
			Type::Null => false,
			Type::Value(kind) => kind.is_valid_value(value),
			Type::Or(a, b) => {
				let a = a.validate_value(value);
				let b = b.validate_value(value);
				return a.or(b);
			}
			Type::Ref(val) => return val.validate_value(value),
		};
		if valid {
			Ok(())
		} else {
			let typ = value.type_name();
			Err(Errors::from(
				format!("value `{value}` of type `{typ}` is not valid {self:?}"),
				Span::default(),
			))
		}
	}

	/// Return the actual type for the a value, disregarding reference types.
	pub fn value(&self) -> &Type {
		match self {
			Type::Ref(val) => &*val,
			_ => self,
		}
	}

	pub fn is_string(&self) -> bool {
		self.value() == &Type::Value(ValueType::Str)
	}

	pub fn is_boolean(&self) -> bool {
		self.value() == &Type::Value(ValueType::Bool)
	}

	pub fn is_int(&self) -> bool {
		self.get_int_type().is_some()
	}

	pub fn get_int_type(&self) -> Option<&IntType> {
		match self.value() {
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
		let result = expr.execute(&mut scope)?.value();
		assert_eq!(result, Value::from(5));

		Ok(())
	}
}
