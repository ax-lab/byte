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

pub mod expr;
pub mod expr_value;
pub mod info;

pub use expr::*;
pub use expr_value::*;
pub use info::*;

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

impl Node {
	pub fn generate_code(&mut self) -> Result<Expr> {
		let output = self.generate_node();
		let scope = self.scope();
		if (output.is_err() && DEBUG_NODES) || DUMP_CODE || scope.program().dump_enabled() {
			println!("\n------ SOURCE ------\n");
			println!("{self}");
			println!("\n--------------------");

			println!("\n------ OUTPUT ------\n");
			if let Ok(output) = &output {
				println!("{output}");
			} else {
				println!("{output:#?}");
			}
			println!("\n--------------------");
		}

		output.at_pos(self.span())
	}

	pub fn as_expr(&self, parent: &Expr) -> Result<Expr> {
		match self.val() {
			NodeValue::Code(expr) => Ok(expr.clone()),
			_ => {
				if parent.info().solve() {
					self.generate_node()
				} else {
					let ctx = Context::get().format_without_span();
					ctx.is_used();
					let error = format!("unresolved node in the expression tree -- `{self}`");
					let error = Errors::from(error, self.span());
					Err(error)
				}
			}
		}
	}

	fn generate_node(&self) -> Result<Expr> {
		let span = self.span();
		let info = Info::new(span.clone());
		let value = match self.val() {
			NodeValue::Code(expr) => expr,
			NodeValue::Raw(list) => match list.len() {
				0 => Expr::Unit(info),
				1 => list[0].generate_node()?,
				_ => {
					let mut errors = Errors::new();
					let mut sequence = Vec::new();
					for node in list.iter() {
						match node.generate_node() {
							Ok(expr) => sequence.push(expr),
							Err(err) => errors.append(&err),
						}
					}
					errors.check()?;
					Expr::Sequence(info, sequence)
				}
			},
			NodeValue::Boolean(value) => Expr::Bool(info, value),
			NodeValue::Token(Token::Integer(value)) => {
				let value = IntValue::new(value, DEFAULT_INT).at_pos(span)?;
				Expr::Int(info, value)
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
				Expr::Float(info, value)
			}
			NodeValue::Null => Expr::Null(info),
			NodeValue::Token(Token::Literal(value)) => Expr::Str(info, value.clone()),
			NodeValue::Group(list) => list.generate_node()?,
			NodeValue::Let(name, offset, list) => {
				let expr = list.generate_node()?;
				let offset = offset;
				Expr::Declare(info, name.clone(), offset, Arc::new(expr))
			}
			NodeValue::If {
				expr: condition,
				if_true,
				if_false,
			} => {
				let condition = condition.generate_node()?;
				let if_true = if_true.generate_node()?;
				let if_false = if let Some(if_false) = if_false {
					if_false.generate_node()?
				} else {
					Expr::Unit(Info::none())
				};
				Expr::Conditional(info, condition.into(), if_true.into(), if_false.into())
			}
			NodeValue::For {
				var,
				offset,
				from,
				to,
				body,
			} => {
				// TODO: `for` should validate types on code generation
				let from = from.generate_node()?;
				let to = to.generate_node()?;
				let body = body.generate_node()?;
				Expr::For(info, var.clone(), offset, from.into(), to.into(), body.into())
			}
			NodeValue::Variable(name, index) => {
				let scope = self.scope();
				if let Some(value) = scope.get(name.clone(), &index) {
					Expr::Variable(
						info,
						name.clone(),
						index,
						value.get_type().because(format!("solving variable `{name}`"), &span)?,
					)
				} else {
					let error = format!("variable `{name}` ({index:?}) does not match any declaration");
					let error = Errors::from(error, span);
					return Err(error);
				}
			}
			NodeValue::Print(expr, tail) => {
				let expr = expr.generate_node()?;
				Expr::Print(info, expr.into(), tail)
			}
			NodeValue::UnaryOp(op, arg) => {
				let arg = arg.generate_node()?;
				let op_impl = op.for_type(&arg.get_type()?)?;
				Expr::Unary(info, op, op_impl, arg.into())
			}
			NodeValue::BinaryOp(op, lhs, rhs) => {
				let lhs = lhs.generate_node()?;
				let rhs = rhs.generate_node()?;
				let op_impl = op.for_types(&lhs.get_type()?, &rhs.get_type()?)?;
				Expr::Binary(info, op, op_impl, lhs.into(), rhs.into())
			}
			NodeValue::Sequence(list) => {
				let mut errors = Errors::new();
				let mut sequence = Vec::new();
				for it in list.iter() {
					match it.generate_node() {
						Ok(expr) => sequence.push(expr),
						Err(err) => errors.append(&err),
					};
					if errors.len() >= MAX_ERRORS {
						break;
					}
				}
				errors.check()?;
				Expr::Sequence(info, sequence)
			}
			NodeValue::Conditional(a, b, c) => {
				let a = a.generate_node()?;
				let b = b.generate_node()?;
				let c = c.generate_node()?;
				Expr::Conditional(info, a.into(), b.into(), c.into())
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

		// TODO: review the need and use of `set_value_inner`
		//
		// Generate_node cannot be mutable without too much fuzz, so we're doing
		// this because it is only advisory anyway (internally it uses a mutex)
		// and our external interface already requires a mut reference.
		//
		// Even though the above is "safe", it still reeks, so maybe review this
		// whole node mutability thing.
		let span = self.span();
		unsafe {
			self.set_value_inner(NodeValue::Code(value.clone()), span);
		}
		Ok(value)
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
		let a = Expr::Int(Info::none(), IntValue::I32(2));
		let b = Expr::Int(Info::none(), IntValue::I32(3));

		let op = BinaryOpImpl::from(OpAdd::for_type(&a.get_type()?).unwrap());

		let expr = Expr::Binary(Info::none(), BinaryOp::Add, op, a.into(), b.into());

		let mut scope = RuntimeScope::new();
		let result = expr.execute(&mut scope)?.into_value();
		assert_eq!(result, Value::from(5));

		Ok(())
	}
}
