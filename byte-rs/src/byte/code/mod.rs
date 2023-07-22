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

pub mod expr_value;
pub use expr_value::*;

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
	pub fn generate_code(&mut self) -> Result<Node> {
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

	fn generate_node(&self) -> Result<Node> {
		let scope = self.scope_handle();
		let span = self.span();
		let value = match self.val() {
			NodeValue::Raw(list) => match list.len() {
				0 => NodeValue::Unit.at(scope, span),
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
					NodeValue::Sequence(sequence.into()).at(scope, span)
				}
			},
			NodeValue::Boolean(..) => self.clone(),
			NodeValue::Token(Token::Integer(value)) => {
				let value = IntValue::new(value, DEFAULT_INT).at_pos(span.clone())?;
				NodeValue::Int(value).at(scope, span)
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
				NodeValue::Float(value).at(scope, span)
			}
			NodeValue::Null => self.clone(),
			NodeValue::Token(Token::Literal(value)) => NodeValue::Str(value.clone()).at(scope, span),
			NodeValue::Group(inner) => inner.generate_node()?,
			NodeValue::Let(name, offset, list) => {
				let expr = list.generate_node()?;
				let offset = offset;
				NodeValue::Let(name.clone(), offset, expr).at(scope, span)
			}
			NodeValue::If {
				expr,
				if_true,
				if_false,
			} => {
				let expr = expr.generate_node()?;
				let if_true = if_true.generate_node()?;
				let if_false = if let Some(if_false) = if_false {
					Some(if_false.generate_node()?)
				} else {
					None
				};
				NodeValue::If {
					expr,
					if_true,
					if_false,
				}
				.at(scope, span)
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
				NodeValue::For {
					var,
					offset,
					from,
					to,
					body,
				}
				.at(scope, span)
			}
			NodeValue::UnresolvedVariable(name, index) => {
				let scope = scope.get();
				if let Some(mut value) = scope.get(name.clone(), &index) {
					let value = value.generate_code()?;
					NodeValue::Variable(
						name.clone(),
						index,
						value.get_type().because(format!("solving variable `{name}`"), &span)?,
					)
					.at(scope.handle(), span)
				} else {
					let error = format!("variable `{name}` ({index:?}) does not match any declaration");
					let error = Errors::from(error, span);
					return Err(error);
				}
			}
			NodeValue::Print(expr, tail) => {
				let expr = expr.generate_node()?;
				NodeValue::Print(expr, tail).at(scope, span)
			}
			NodeValue::UnaryOp(op, op_impl, arg) => {
				if op_impl.is_some() {
					self.clone()
				} else {
					let arg = arg.generate_node()?;
					let op_impl = op.for_type(&arg.get_type()?)?;
					NodeValue::UnaryOp(op, Some(op_impl), arg).at(scope, span)
				}
			}
			NodeValue::BinaryOp(op, op_impl, lhs, rhs) => {
				if op_impl.is_some() {
					self.clone()
				} else {
					let lhs = lhs.generate_node()?;
					let rhs = rhs.generate_node()?;
					let op_impl = op.for_types(&lhs.get_type()?, &rhs.get_type()?)?;
					NodeValue::BinaryOp(op, Some(op_impl), lhs, rhs).at(scope, span)
				}
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
				NodeValue::Sequence(sequence.into()).at(scope, span)
			}
			NodeValue::Conditional(a, b, c) => {
				let a = a.generate_node()?;
				let b = b.generate_node()?;
				let c = c.generate_node()?;
				NodeValue::Conditional(a, b, c).at(scope, span)
			}
			value => {
				let mut error = format!("cannot generate code for {}", value.short_repr());
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
// Tests
//====================================================================================================================//

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn basic_eval() -> Result<()> {
		let compiler = Compiler::new();
		let program = compiler.new_program();
		let scope = program.root_scope().handle();
		let a = NodeValue::Int(IntValue::I32(2)).at(scope.clone(), Span::default());
		let b = NodeValue::Int(IntValue::I32(3)).at(scope.clone(), Span::default());

		let op = BinaryOpImpl::from(OpAdd::for_type(&a.get_type()?).unwrap());

		let expr = NodeValue::BinaryOp(BinaryOp::Add, Some(op), a, b).at(scope, Span::default());

		let mut scope = RuntimeScope::new();
		let result = expr.execute(&mut scope)?.into_value();
		assert_eq!(result, Value::from(5));

		Ok(())
	}
}
