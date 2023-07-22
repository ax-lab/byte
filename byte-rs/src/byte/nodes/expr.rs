use super::*;

// TODO: differentiate between a resolved and unresolved node for codegen

/// Enumeration of all available language elements.
///
/// Nodes relate to the source code, representing language constructs of all
/// levels, from files, raw text, and tokens, all the way to fully fledged
/// definitions.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum Expr {
	Token(Token),
	//----[ Basic values ]----------------------------------------------------//
	Unit,
	Null,
	Never,
	Str(StringValue),
	Int(IntValue),
	Float(FloatValue),
	Boolean(bool),
	Variable(Symbol, CodeOffset, Type),
	//----[ Structural ]------------------------------------------------------//
	Raw(Arc<Vec<Node>>),
	Sequence(Arc<Vec<Node>>),
	Group(Node),
	Block(Node, Node),
	//----[ Logic ]-----------------------------------------------------------//
	If {
		expr: Node,
		if_true: Node,
		if_false: Option<Node>,
	},
	For {
		var: Symbol,
		offset: CodeOffset,
		from: Node,
		to: Node,
		body: Node,
	},
	//----[ AST ]-------------------------------------------------------------//
	Let(Symbol, CodeOffset, Node),
	UnaryOp(UnaryOp, Option<UnaryOpImpl>, Node),
	BinaryOp(BinaryOp, Option<BinaryOpImpl>, Node, Node),
	UnresolvedVariable(Symbol, CodeOffset),
	Print(Node, &'static str),
	Conditional(Node, Node, Node),
	// TODO: Apply(Info, Func, Vec<Node>),
}

impl Expr {
	pub fn get_type(self) -> Result<Type> {
		let typ = match self {
			Expr::Never => Type::Never,
			Expr::Unit => Type::Unit,
			Expr::Null => Type::Null,
			Expr::Let(.., expr) => expr.get_type()?,
			Expr::Boolean(..) => Type::Bool,
			Expr::Str(..) => Type::String,
			Expr::Int(.., int) => Type::Int(int.get_type()),
			Expr::For { .. } => Type::Unit,
			Expr::Float(.., float) => Type::Float(float.get_type()),
			Expr::Variable(.., kind) => Type::Ref(kind.clone().into()),
			Expr::Print(..) => Type::Unit,
			Expr::UnaryOp(_, op, ..) => {
				if let Some(op) = op {
					op.get().get_type()
				} else {
					Type::Unknown
				}
			}
			Expr::BinaryOp(_, op, ..) => {
				if let Some(op) = op {
					op.get().get_type()
				} else {
					Type::Unknown
				}
			}
			Expr::Sequence(.., list) => list.last().map(|x| x.get_type()).unwrap_or_else(|| Ok(Type::Unit))?,
			Expr::Conditional(_, a, b) => {
				let a = a.get_type()?;
				let b = b.get_type()?;
				if a == b {
					a
				} else {
					Type::Or(a.into(), b.into())
				}
			}
			_ => Type::Unknown,
		};
		Ok(typ)
	}

	pub fn at(self, scope: ScopeHandle, span: Span) -> Node {
		Node::new(self, scope, at(span))
	}

	pub fn symbol(&self) -> Option<Symbol> {
		let symbol = match self {
			Expr::Token(Token::Word(symbol)) => symbol,
			Expr::Token(Token::Symbol(symbol)) => symbol,
			_ => return None,
		};
		Some(symbol.clone())
	}

	pub fn len(&self) -> usize {
		self.children().len()
	}

	pub fn iter(&self) -> impl Iterator<Item = &Node> {
		self.children().into_iter()
	}

	pub fn get(&self, index: usize) -> Option<&Node> {
		self.children().get(index).cloned()
	}

	pub fn children(&self) -> Vec<&Node> {
		match self {
			Expr::Token(_) => vec![],
			Expr::Null => vec![],
			Expr::Boolean(_) => vec![],
			Expr::Unit => vec![],
			Expr::Never => vec![],
			Expr::Str(_) => vec![],
			Expr::Int(_) => vec![],
			Expr::Float(_) => vec![],
			Expr::Variable(_, _, _) => vec![],
			Expr::Raw(ls) => ls.iter().map(|x| x).collect(),
			Expr::Sequence(ls) => ls.iter().map(|x| x).collect(),
			Expr::Group(it) => vec![it],
			Expr::Block(head, body) => vec![head, body],
			Expr::If {
				expr,
				if_true,
				if_false,
			} => {
				let mut out = vec![expr, if_true];
				if let Some(ref if_false) = if_false {
					out.push(if_false);
				}
				out
			}
			Expr::For { from, to, body, .. } => vec![from, to, body],
			Expr::Let(.., expr) => vec![expr],
			Expr::UnaryOp(_, _, expr) => vec![expr],
			Expr::BinaryOp(BinaryOp::Member, _, lhs, rhs) => {
				if rhs.as_identifier().is_some() {
					vec![lhs]
				} else {
					vec![lhs, rhs]
				}
			}
			Expr::BinaryOp(_, _, lhs, rhs) => vec![lhs, rhs],
			Expr::UnresolvedVariable(..) => vec![],
			Expr::Print(expr, _) => vec![expr],
			Expr::Conditional(cond, t, f) => vec![cond, t, f],
		}
	}

	pub fn get_dependencies<P: FnMut(&Node)>(&self, mut output: P) {
		for it in self.children() {
			output(it);
		}
	}

	pub fn short_repr(&self) -> String {
		let short = |title, list| {
			let span = Span::from_node_vec(list);
			let ctx = Context::get();
			ctx.with_format(ctx.format().with_mode(Mode::Minimal), || format!("<{title} {span}>"))
		};
		match self {
			Expr::Token(..) => format!("{self}"),
			Expr::Null => format!("{self}"),
			Expr::Boolean(..) => format!("{self}"),
			Expr::Unit => format!("{self}"),
			Expr::Never => format!("{self}"),
			Expr::Str(..) => format!("{self}"),
			Expr::Int(..) => format!("{self}"),
			Expr::Float(..) => format!("{self}"),
			Expr::Variable(..) => format!("{self}"),
			Expr::Raw(list) => short("raw", list),
			Expr::Sequence(list) => short("seq", list),
			Expr::Group(value) => {
				let repr = value.short_repr();
				format!("{{ {repr} }}")
			}
			Expr::Block(head, ..) => {
				let head = head.short_repr();
				format!("<block {head}...>")
			}
			Expr::If { expr, .. } => {
				let expr = expr.short_repr();
				format!("<if {expr}...>")
			}
			Expr::For { var, from, to, .. } => format!("<for {var} in {from}..{to}>"),
			Expr::Let(name, _, expr) => format!("<let {name} = {}>", expr.short_repr()),
			Expr::UnaryOp(op, _, arg) => format!("({op} {})", arg.short_repr()),
			Expr::BinaryOp(op, _, lhs, rhs) => format!("({op} {} {})", lhs.short_repr(), rhs.short_repr()),
			Expr::UnresolvedVariable(name, _) => format!("<var {name}>"),
			Expr::Print(expr, _) => format!("<print {}>", expr.short_repr()),
			Expr::Conditional(a, b, c) => {
				format!("<{} ? {} : {}>", a.short_repr(), b.short_repr(), c.short_repr())
			}
		}
	}
}

impl Display for Expr {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		let ctx = Context::get().format_without_span();
		ctx.is_used();

		match self {
			Expr::Token(token) => write!(f, "`{token}`"),
			Expr::Null => write!(f, "null"),
			Expr::Boolean(value) => write!(f, "{value}"),
			Expr::Unit => write!(f, "()"),
			Expr::Never => write!(f, "(!)"),
			Expr::Str(value) => write!(f, "{value:?}"),
			Expr::Int(value) => write!(f, "{value:?}"),
			Expr::Float(value) => write!(f, "{value:?}"),
			Expr::Variable(name, at, kind) => write!(f, "({name}{at:?} as {kind})"),
			Expr::Raw(nodes) => match nodes.len() {
				0 => write!(f, "[]"),
				1 => {
					let output = format!("{}", nodes[0]);
					if output.contains('\n') || output.len() > 50 {
						write!(f.indented(), "[\n{output}")?;
						write!(f, "\n]")
					} else {
						write!(f, "[{output}]")
					}
				}
				_ => {
					let ctx = ctx.format_with_span();
					ctx.is_used();
					write!(f, "[# raw:")?;
					let mut out = f.indented();
					for (n, it) in nodes.iter().enumerate() {
						write!(out, "\n# {n}:")?;
						write!(out.indented(), "\n{it}")?;
					}
					write!(f, "\n]")
				}
			},
			Expr::Sequence(nodes) => {
				let ctx = ctx.format_with_span();
				ctx.is_used();
				write!(f, "# sequence:")?;
				let mut out = f.indented();
				for (n, it) in nodes.iter().enumerate() {
					write!(out, "\n# {n}:")?;
					write!(out.indented(), "\n{it}")?;
				}
				Ok(())
			}
			Expr::Group(value) => write!(f, "{{ {value} }}"),
			Expr::Block(head, body) => {
				write!(f, "block({head}:")?;
				write!(f.indented(), "\n{body}")?;
				write!(f, "\n)")
			}
			Expr::If {
				expr,
				if_true,
				if_false,
			} => {
				write!(f, "if ({expr}) {{")?;
				write!(f.indented(), "\n{if_true}")?;
				if let Some(if_false) = if_false {
					write!(f, "\n}} else {{")?;
					write!(f.indented(), "\n{if_false}")?;
				}
				write!(f, "\n}}")
			}
			Expr::For {
				var,
				offset,
				from,
				to,
				body,
			} => {
				write!(f, "for {var}{offset} in {from}..{to} {{")?;
				write!(f.indented(), "\n{body}")?;
				write!(f, "\n}}")
			}
			Expr::Let(name, offset, expr) => {
				let offset = offset.value();
				write!(f, "let {name}{offset} = {expr}")
			}
			Expr::UnaryOp(op, _, arg) => write!(f, "({op} {arg})"),
			Expr::BinaryOp(op, _, lhs, rhs) => write!(f, "({lhs} {op} {rhs})"),
			Expr::UnresolvedVariable(var, offset) => {
				let offset = offset.value();
				write!(f, "{var}{offset}")
			}
			Expr::Print(expr, _) => write!(f, "print({expr})"),
			Expr::Conditional(cond, a, b) => write!(f, "{cond} ? {a} : {b}"),
		}
	}
}
