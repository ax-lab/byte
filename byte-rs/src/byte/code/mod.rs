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

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum CodeOffset {
	Static,
	At(usize),
}

impl CodeOffset {
	pub fn value(&self) -> usize {
		match self {
			CodeOffset::Static => 0,
			CodeOffset::At(offset) => *offset,
		}
	}
}

impl Display for CodeOffset {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		match self {
			CodeOffset::Static => write!(f, "static scope"),
			CodeOffset::At(offset) => write!(f, "offset {offset}"),
		}
	}
}

impl Debug for CodeOffset {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		write!(f, ".{}", self.value())
	}
}

// TODO: merge this or include it in the Scope
pub struct CodeContext {
	declares: HashMap<(Symbol, CodeOffset), Type>,
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
		let output = self.generate_node(context);
		if (output.is_err() && DEBUG_NODES) || DUMP_CODE || context.dump_code {
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

	pub fn as_expr(&self) -> Result<Expr> {
		/*
			TODO: figure out Expr::Node

			Currently, Expr::Node is only used by the scope declarations, so
			the value never actually makes it into the expression tree.

			If it did, it would actually cause an error here because the node
			is embedded in the expression, but never actually makes it as code.

			One solution would be for `generate_node` to rewrite the node as
			an `NodeValue::Code` when solving.

			The other would be for this method to actually try to solve the
			node code, which requires merging CodeContext and Scope.
		*/
		if let NodeValue::Code(expr) = self.val() {
			Ok(expr.clone())
		} else {
			let error = format!("unresolved node in the expression tree: {self}");
			let error = Errors::from(error, self.span());
			Err(error)
		}
	}

	fn generate_node(&self, context: &mut CodeContext) -> Result<Expr> {
		let span = self.span();
		let info = Info::new(span.clone());
		let value = match self.val() {
			NodeValue::Code(expr) => expr,
			NodeValue::Raw(list) => match list.len() {
				0 => Expr::Unit(info),
				1 => list[0].generate_node(context)?,
				_ => {
					let mut errors = Errors::new();
					let mut sequence = Vec::new();
					for node in list.iter() {
						match node.generate_node(context) {
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
			NodeValue::Group(list) => list.generate_node(context)?,
			NodeValue::Let(name, offset, list) => {
				let expr = list.generate_node(context)?;
				let kind = expr.get_type()?;
				let offset = offset;
				context.declares.insert((name.clone(), offset), kind);
				Expr::Declare(info, name.clone(), offset, Arc::new(expr))
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
				let from = from.generate_node(context)?;
				let to = to.generate_node(context)?;
				context.declares.insert((var.clone(), offset), from.get_type()?);
				let body = body.generate_node(context)?;
				Expr::For(info, var.clone(), offset, from.into(), to.into(), body.into())
			}
			NodeValue::Variable(name, index) => {
				if let Some(kind) = context.declares.get(&(name.clone(), index)) {
					Expr::Variable(info, name.clone(), index, kind.clone())
				} else {
					let error = format!("variable `{name}` ({index:?}) does not match any declaration");
					let error = Errors::from(error, span);
					return Err(error);
				}
			}
			NodeValue::Print(expr, tail) => {
				let expr = expr.generate_node(context)?;
				Expr::Print(info, expr.into(), tail)
			}
			NodeValue::UnaryOp(op, arg) => {
				let arg = arg.generate_node(context)?;
				let op_impl = op.for_type(&arg.get_type()?)?;
				Expr::Unary(info, op, op_impl, arg.into())
			}
			NodeValue::BinaryOp(op, lhs, rhs) => {
				let lhs = lhs.generate_node(context)?;
				let rhs = rhs.generate_node(context)?;
				let op_impl = op.for_types(&lhs.get_type()?, &rhs.get_type()?)?;
				Expr::Binary(info, op, op_impl, lhs.into(), rhs.into())
			}
			NodeValue::Sequence(list) => {
				let mut errors = Errors::new();
				let mut sequence = Vec::new();
				for it in list.iter() {
					match it.generate_node(context) {
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
				let a = a.generate_node(context)?;
				let b = b.generate_node(context)?;
				let c = c.generate_node(context)?;
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
		Ok(value)
	}
}

//====================================================================================================================//
// Expressions
//====================================================================================================================//

/*
	TODO: add the ability to have nodes in Expr.

	- add an expression that is just a proxy for a node, the counterpart for Node::Code
	- this will allow to use Expr directly in the node tree, eliminating duplicated Node/Expr
	- the proxy node will have to be added as a separate entry to the program resolve list
	  - alternatively, Expr can have a way of iterating nodes, but that is iffy
	- the node should be fully solved by the time the expression is being generated
*/

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Info {
	id: Id,
	span: Span,
}

impl Info {
	pub fn none() -> Self {
		Self {
			id: id(),
			span: Span::default(),
		}
	}
	pub fn new(span: Span) -> Self {
		Self { id: id(), span }
	}

	pub fn id(&self) -> Id {
		self.id
	}

	pub fn span(&self) -> &Span {
		&self.span
	}
}

/// Enumeration of builtin root expressions.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum Expr {
	Never(Info),
	Unit(Info),
	Null(Info),
	Node(Info, Node),
	Declare(Info, Symbol, CodeOffset, Arc<Expr>),
	Conditional(Info, Arc<Expr>, Arc<Expr>, Arc<Expr>),
	For(Info, Symbol, CodeOffset, Arc<Expr>, Arc<Expr>, Arc<Expr>),
	Bool(Info, bool),
	Str(Info, StringValue),
	Int(Info, IntValue),
	Float(Info, FloatValue),
	Variable(Info, Symbol, CodeOffset, Type),
	Print(Info, Arc<Expr>, &'static str),
	Unary(Info, UnaryOp, UnaryOpImpl, Arc<Expr>),
	Binary(Info, BinaryOp, BinaryOpImpl, Arc<Expr>, Arc<Expr>),
	Sequence(Info, Vec<Expr>),
}

impl Expr {
	pub fn from_node(node: Node) -> Self {
		let info = Info::new(node.span());
		Expr::Node(info, node)
	}

	pub fn info(&self) -> &Info {
		match self {
			Expr::Never(info) => info,
			Expr::Unit(info) => info,
			Expr::Null(info) => info,
			Expr::Node(info, ..) => info,
			Expr::Declare(info, ..) => info,
			Expr::Conditional(info, ..) => info,
			Expr::For(info, ..) => info,
			Expr::Bool(info, ..) => info,
			Expr::Str(info, ..) => info,
			Expr::Int(info, ..) => info,
			Expr::Float(info, ..) => info,
			Expr::Variable(info, ..) => info,
			Expr::Print(info, ..) => info,
			Expr::Unary(info, ..) => info,
			Expr::Binary(info, ..) => info,
			Expr::Sequence(info, ..) => info,
		}
	}

	pub fn span(&self) -> &Span {
		self.info().span()
	}

	pub fn get_type(&self) -> Result<Type> {
		let typ = match self {
			Expr::Never(..) => Type::Never,
			Expr::Unit(..) => Type::Unit,
			Expr::Null(..) => Type::Null,
			Expr::Node(.., node) => {
				// TODO: depending on how as_expr is implemented, this could be bad
				return node.as_expr()?.get_type();
			}
			Expr::Declare(.., expr) => expr.get_type()?,
			Expr::Bool(..) => Type::Bool,
			Expr::Str(..) => Type::String,
			Expr::Int(.., int) => Type::Int(int.get_type()),
			Expr::For(..) => Type::Unit,
			Expr::Float(.., float) => Type::Float(float.get_type()),
			Expr::Variable(.., kind) => Type::Ref(kind.clone().into()),
			Expr::Print(..) => Type::Unit,
			Expr::Unary(_, _, op, ..) => op.get().get_type(),
			Expr::Binary(_, _, op, ..) => op.get().get_type(),
			Expr::Sequence(.., list) => list.last().map(|x| x.get_type()).unwrap_or_else(|| Ok(Type::Unit))?,
			Expr::Conditional(_, _, a, b) => {
				let a = a.get_type()?;
				let b = b.get_type()?;
				if a == b {
					a
				} else {
					Type::Or(a.into(), b.into())
				}
			}
		};
		Ok(typ)
	}

	pub fn execute(&self, scope: &mut RuntimeScope) -> Result<ExprValue> {
		match self {
			Expr::Never(..) => {
				let error = format!("never expression cannot be evaluated");
				Err(Errors::from(error, self.span().clone()))
			}
			Expr::Node(..) => {
				// TODO: allow runtime generation of nodes
				let error = format!("Expr::Node should never make it past codegen");
				Err(Errors::from(error, self.span().clone()))
			}
			Expr::Unit(..) => Ok(Value::from(()).into()),
			Expr::Null(..) => Ok(Value::Null.into()),
			Expr::Declare(_, name, offset, expr) => {
				let value = expr.execute(scope)?;
				scope.set(name.clone(), *offset, value.clone().into());
				Ok(value)
			}
			Expr::Bool(_, value) => Ok(Value::from(*value).into()),
			Expr::Str(_, value) => Ok(Value::from(value.to_string()).into()),
			Expr::Int(_, value) => Ok(Value::from(value.clone()).into()),
			Expr::Float(_, value) => Ok(Value::from(value.clone()).into()),
			Expr::Variable(_, name, offset, ..) => match scope.get(name, *offset).cloned() {
				Some(value) => Ok(ExprValue::Variable(name.clone(), offset.clone(), value)),
				None => Err(Errors::from(format!("variable {name} not set"), self.span().clone())),
			},
			Expr::Print(_, expr, tail) => {
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
			Expr::Unary(_, _, op, arg) => op.get().execute(scope, &arg),
			Expr::Binary(_, _, op, lhs, rhs) => op.get().execute(scope, lhs, rhs),
			Expr::Sequence(_, list) => {
				let mut value = Value::from(()).into();
				for it in list.iter() {
					value = it.execute(scope)?;
				}
				Ok(value)
			}
			Expr::Conditional(_, cond, a, b) => {
				let cond = cond.execute(scope)?;
				let cond = cond.value().bool()?;
				if cond {
					a.execute(scope)
				} else {
					b.execute(scope)
				}
			}
			Expr::For(_, var, offset, from, to, body) => {
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

				let mut cur = from_value;
				loop {
					let value = Value::from(cur).cast_to(&from_type, NumericConversion::None)?;
					scope.set(var.clone(), *offset, value);
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
			Expr::Sequence(_, list) => output = list.clone(),
			expr => output.push(expr.clone()),
		}
		output
	}
}

#[derive(Clone, Debug)]
pub enum ExprValue {
	Value(Value),
	Variable(Symbol, CodeOffset, Value),
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

impl Display for Expr {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		match self {
			Expr::Never(..) => write!(f, "(!)"),
			Expr::Unit(..) => write!(f, "()"),
			Expr::Null(..) => write!(f, "null"),
			Expr::Node(.., node) => write!(f, "<<{node}>>"),
			Expr::Declare(_, name, at, expr) => write!(
				f,
				"decl {name}{at:?}: {} = {expr}",
				match expr.get_type() {
					Ok(typ) => format!("{typ}"),
					Err(_) => format!("!?"),
				}
			),
			Expr::Conditional(_, cond, a, b) => {
				write!(f, "if {cond} {{")?;
				write!(f.indented(), "\n{a}")?;
				write!(f, "\n}} else {{")?;
				write!(f.indented(), "\n{b}")?;
				write!(f, "\n}}")
			}
			Expr::For(_, name, at, from, to, expr) => {
				write!(f, "for {name}{at:?} in {from}..{to} {{")?;
				write!(f.indented(), "\n{expr}")?;
				write!(f, "\n}}")
			}
			Expr::Bool(_, value) => write!(f, "{value}"),
			Expr::Str(_, value) => write!(f, "{value:?}"),
			Expr::Int(_, value) => write!(f, "{value:?}"),
			Expr::Float(_, value) => write!(f, "{value:?}"),
			Expr::Variable(_, name, at, kind) => write!(f, "<{name}{at:?} -- {kind}>"),
			Expr::Print(_, expr, _) => write!(f, "print({expr})"),
			Expr::Unary(_, op, _, arg) => write!(f, "{op}({arg})"),
			Expr::Binary(_, op, _, lhs, rhs) => write!(f, "({lhs}) {op} ({rhs})"),
			Expr::Sequence(_, list) => match list.len() {
				0 => write!(f, "{{ }}"),
				1 => write!(f, "{{ {} }}", list[0]),
				_ => {
					write!(f, "{{")?;
					for it in list.iter() {
						write!(f.indented(), "\n{it};")?;
					}
					write!(f, "\n}}")
				}
			},
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
