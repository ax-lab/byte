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

pub mod op;
pub mod op_add;
pub mod op_and;
pub mod op_assign;
pub mod op_compare_equal;
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

pub use op::*;
pub use op_add::*;
pub use op_and::*;
pub use op_assign::*;
pub use op_compare_equal::*;
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

use super::*;

const DEBUG_NODES: bool = false;

pub struct CodeContext {
	declares: HashMap<(Symbol, Option<usize>), Type>,
	dump_code: bool,
}

impl CodeContext {
	pub fn new() -> Self {
		Self {
			declares: Default::default(),
			dump_code: false,
		}
	}

	pub fn dump_code(&mut self) {
		self.dump_code = true;
	}
}

impl Node {
	pub fn generate_code(&self, context: &mut CodeContext) -> Result<Expr> {
		let mut errors = Errors::new();

		let output = self.generate_node(context).handle(&mut errors);
		if (errors.len() > 0 && DEBUG_NODES) || DUMP_CODE || context.dump_code {
			println!("\n------ SOURCE ------\n");
			println!("{self}");
			println!("\n--------------------");

			println!("\n------ OUTPUT ------\n");
			println!("{output:#?}");
			println!("\n--------------------");
		}

		Ok(output).unless(errors).at_pos(self.span())
	}

	fn generate_node(&self, context: &mut CodeContext) -> Result<Expr> {
		let span = self.span();
		let value = match self.val() {
			NodeValue::Raw(list) => match list.len() {
				0 => Expr::Unit,
				1 => list[0].generate_node(context)?,
				_ => {
					let mut errors = Errors::new();
					let mut sequence = Vec::new();
					for node in list.iter() {
						let expr = node.generate_node(context).handle(&mut errors);
						sequence.push(expr);
					}
					Expr::Sequence(sequence)
				}
			},
			NodeValue::Boolean(value) => Expr::Bool(value),
			NodeValue::Token(Token::Integer(value)) => {
				let value = IntValue::new(value, DEFAULT_INT).at_pos(span)?;
				Expr::Int(value)
			}
			NodeValue::Token(Token::Float(value)) => {
				let value: f64 = match value.as_str().parse() {
					Ok(value) => value,
					Err(err) => {
						let error = Errors::from(format!("not a valid float: {err}"), span);
						return Err(error);
					}
				};
				let value = FloatValue::new(value, FloatType::F64);
				Expr::Float(value)
			}
			NodeValue::Null => Expr::Null,
			NodeValue::Token(Token::Literal(value)) => Expr::Str(value.clone()),
			NodeValue::Group(list) => list.generate_node(context)?,
			NodeValue::Let(name, offset, list) => {
				let expr = list.generate_node(context)?;
				let kind = expr.get_type();
				let offset = offset;
				context.declares.insert((name.clone(), offset), kind);
				Expr::Declare(name.clone(), offset, Arc::new(expr))
			}
			NodeValue::If {
				expr: condition,
				if_true,
				if_false,
			} => {
				let condition = condition.generate_node(context)?;
				let if_true = if_true.generate_node(context)?;
				let if_false = if let Some(if_false) = if_false {
					if_false.generate_node(context)?
				} else {
					Expr::Unit
				};
				Expr::Conditional(condition.into(), if_true.into(), if_false.into())
			}
			NodeValue::For {
				var,
				offset,
				from,
				to,
				body,
			} => {
				// TODO: `for` should validate types on code generation
				let from = from.generate_node(context)?;
				let to = to.generate_node(context)?;
				context.declares.insert((var.clone(), Some(offset)), from.get_type());
				let body = body.generate_node(context)?;
				Expr::For(var.clone(), offset, from.into(), to.into(), body.into())
			}
			NodeValue::Variable(name, index) => {
				if let Some(kind) = context.declares.get(&(name.clone(), index)) {
					Expr::Variable(name.clone(), index, kind.clone())
				} else {
					let error = format!("variable `{name}` ({index:?}) does not match any declaration");
					let error = Errors::from(error, span);
					return Err(error);
				}
			}
			NodeValue::Print(expr, tail) => {
				let expr = expr.generate_node(context)?;
				Expr::Print(expr.into(), tail)
			}
			NodeValue::UnaryOp(op, arg) => {
				let arg = arg.generate_node(context)?;
				let op = op.for_type(&arg.get_type())?;
				Expr::Unary(op, arg.into())
			}
			NodeValue::BinaryOp(op, lhs, rhs) => {
				let lhs = lhs.generate_node(context)?;
				let rhs = rhs.generate_node(context)?;
				let op = op.for_types(&lhs.get_type(), &rhs.get_type())?;
				Expr::Binary(op, lhs.into(), rhs.into())
			}
			NodeValue::Sequence(list) => {
				let mut errors = Errors::new();
				let mut sequence = Vec::new();
				for it in list.iter() {
					let it = it.generate_node(context).handle(&mut errors);
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
			NodeValue::Conditional(a, b, c) => {
				let a = a.generate_node(context)?;
				let b = b.generate_node(context)?;
				let c = c.generate_node(context)?;
				Expr::Conditional(a.into(), b.into(), c.into())
			}
			value => {
				let mut error = format!("cannot generate code for `{value:?}`");
				{
					let mut output = error.indented();
					let _ = write!(output, "\n\n");
					let _ = write!(output, "{self:?}");
				}
				let error = Errors::from(error, span);
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
#[derive(Clone, Debug, Default)]
pub enum Expr {
	#[default]
	Never,
	Unit,
	Null,
	Declare(Symbol, Option<usize>, Arc<Expr>),
	Conditional(Arc<Expr>, Arc<Expr>, Arc<Expr>),
	For(Symbol, usize, Arc<Expr>, Arc<Expr>, Arc<Expr>),
	Bool(bool),
	Str(StringValue),
	Int(IntValue),
	Float(FloatValue),
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
			Expr::Bool(..) => Type::Bool,
			Expr::Str(..) => Type::String,
			Expr::Int(int) => Type::Int(int.get_type()),
			Expr::For(..) => Type::Unit,
			Expr::Float(float) => Type::Float(float.get_type()),
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
			Expr::Null => Ok(Value::Null.into()),
			Expr::Declare(name, offset, expr) => {
				let value = expr.execute(scope)?;
				scope.set(name.clone(), *offset, value.clone().into());
				Ok(value)
			}
			Expr::Bool(value) => Ok(Value::from(*value).into()),
			Expr::Str(value) => Ok(Value::from(value.to_string()).into()),
			Expr::Int(value) => Ok(Value::from(value.clone()).into()),
			Expr::Float(value) => Ok(Value::from(value.clone()).into()),
			Expr::Variable(name, index, ..) => match scope.get(name, *index).cloned() {
				Some(value) => Ok(ExprValue::Variable(name.clone(), index.clone(), value)),
				None => Err(Errors::from(format!("variable {name} not set"), Span::default())),
			},
			Expr::Print(expr, tail) => {
				let list = expr.as_sequence();
				let mut values = Vec::new();
				for expr in list.iter() {
					let value = expr.execute(scope)?.into_value();
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
				let cond = cond.value().bool()?;
				if cond {
					a.execute(scope)
				} else {
					b.execute(scope)
				}
			}
			Expr::For(var, offset, from, to, body) => {
				let from_value = from.execute(scope)?;
				let from_value = from_value.value();
				let from_type = from_value.get_type();
				let from_value = from_value.int_value(&IntType::I128, NumericConversion::None)?;
				let from_value = from_value.signed();

				let to_value = to.execute(scope)?;
				let to_value = to_value.value();
				let to_value = to_value.int_value(&IntType::I128, NumericConversion::None)?;
				let to_value = to_value.signed();

				let step = if from_value <= to_value { 1 } else { -1 };

				let var_offset = Some(*offset);
				let mut cur = from_value;
				loop {
					let value = Value::from(cur).cast_to(&from_type, NumericConversion::None)?;
					scope.set(var.clone(), var_offset, value);
					body.execute(scope)?;
					if cur == to_value {
						break;
					}
					cur += step;
				}
				Ok(Value::Unit.into())
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
	pub fn value(&self) -> &Value {
		match self {
			ExprValue::Value(ref value) => value,
			ExprValue::Variable(.., ref value) => value,
		}
	}

	pub fn into_value(self) -> Value {
		match self {
			ExprValue::Value(value) => value,
			ExprValue::Variable(.., value) => value,
		}
	}
}

impl From<ExprValue> for Value {
	fn from(expr_value: ExprValue) -> Self {
		expr_value.value().clone()
	}
}

impl From<Value> for ExprValue {
	fn from(value: Value) -> Self {
		ExprValue::Value(value)
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
		let a = Expr::Int(IntValue::I32(2));
		let b = Expr::Int(IntValue::I32(3));

		let op = BinaryOpImpl::from(OpAdd::for_type(&a.get_type()).unwrap());

		let expr = Expr::Binary(op, a.into(), b.into());

		let mut scope = RuntimeScope::new();
		let result = expr.execute(&mut scope)?.into_value();
		assert_eq!(result, Value::from(5));

		Ok(())
	}
}
