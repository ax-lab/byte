use super::*;

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
				return node
					.as_expr(self)
					.because("get node expr type", self.span())?
					.get_type();
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
				None => Err(Errors::from(format!("variable `{name}` not set"), self.span().clone())),
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
