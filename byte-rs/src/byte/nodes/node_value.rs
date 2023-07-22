use super::*;

// TODO: differentiate between a resolved and unresolved node for codegen

/// Enumeration of all available language elements.
///
/// Nodes relate to the source code, representing language constructs of all
/// levels, from files, raw text, and tokens, all the way to fully fledged
/// definitions.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum NodeValue {
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

impl NodeValue {
	pub fn get_type(self) -> Result<Type> {
		let typ = match self {
			NodeValue::Never => Type::Never,
			NodeValue::Unit => Type::Unit,
			NodeValue::Null => Type::Null,
			NodeValue::Let(.., expr) => expr.get_type()?,
			NodeValue::Boolean(..) => Type::Bool,
			NodeValue::Str(..) => Type::String,
			NodeValue::Int(.., int) => Type::Int(int.get_type()),
			NodeValue::For { .. } => Type::Unit,
			NodeValue::Float(.., float) => Type::Float(float.get_type()),
			NodeValue::Variable(.., kind) => Type::Ref(kind.clone().into()),
			NodeValue::Print(..) => Type::Unit,
			NodeValue::UnaryOp(_, op, ..) => {
				if let Some(op) = op {
					op.get().get_type()
				} else {
					Type::Unknown
				}
			}
			NodeValue::BinaryOp(_, op, ..) => {
				if let Some(op) = op {
					op.get().get_type()
				} else {
					Type::Unknown
				}
			}
			NodeValue::Sequence(.., list) => list.last().map(|x| x.get_type()).unwrap_or_else(|| Ok(Type::Unit))?,
			NodeValue::Conditional(_, a, b) => {
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
			NodeValue::Token(Token::Word(symbol)) => symbol,
			NodeValue::Token(Token::Symbol(symbol)) => symbol,
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
			NodeValue::Token(_) => vec![],
			NodeValue::Null => vec![],
			NodeValue::Boolean(_) => vec![],
			NodeValue::Unit => vec![],
			NodeValue::Never => vec![],
			NodeValue::Str(_) => vec![],
			NodeValue::Int(_) => vec![],
			NodeValue::Float(_) => vec![],
			NodeValue::Variable(_, _, _) => vec![],
			NodeValue::Raw(ls) => ls.iter().map(|x| x).collect(),
			NodeValue::Sequence(ls) => ls.iter().map(|x| x).collect(),
			NodeValue::Group(it) => vec![it],
			NodeValue::Block(head, body) => vec![head, body],
			NodeValue::If {
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
			NodeValue::For { from, to, body, .. } => vec![from, to, body],
			NodeValue::Let(.., expr) => vec![expr],
			NodeValue::UnaryOp(_, _, expr) => vec![expr],
			NodeValue::BinaryOp(BinaryOp::Member, _, lhs, rhs) => {
				if rhs.as_identifier().is_some() {
					vec![lhs]
				} else {
					vec![lhs, rhs]
				}
			}
			NodeValue::BinaryOp(_, _, lhs, rhs) => vec![lhs, rhs],
			NodeValue::UnresolvedVariable(..) => vec![],
			NodeValue::Print(expr, _) => vec![expr],
			NodeValue::Conditional(cond, t, f) => vec![cond, t, f],
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
			NodeValue::Token(..) => format!("{self}"),
			NodeValue::Null => format!("{self}"),
			NodeValue::Boolean(..) => format!("{self}"),
			NodeValue::Unit => format!("{self}"),
			NodeValue::Never => format!("{self}"),
			NodeValue::Str(..) => format!("{self}"),
			NodeValue::Int(..) => format!("{self}"),
			NodeValue::Float(..) => format!("{self}"),
			NodeValue::Variable(..) => format!("{self}"),
			NodeValue::Raw(list) => short("raw", list),
			NodeValue::Sequence(list) => short("seq", list),
			NodeValue::Group(value) => {
				let repr = value.short_repr();
				format!("{{ {repr} }}")
			}
			NodeValue::Block(head, ..) => {
				let head = head.short_repr();
				format!("<block {head}...>")
			}
			NodeValue::If { expr, .. } => {
				let expr = expr.short_repr();
				format!("<if {expr}...>")
			}
			NodeValue::For { var, from, to, .. } => format!("<for {var} in {from}..{to}>"),
			NodeValue::Let(name, _, expr) => format!("<let {name} = {}>", expr.short_repr()),
			NodeValue::UnaryOp(op, _, arg) => format!("({op} {})", arg.short_repr()),
			NodeValue::BinaryOp(op, _, lhs, rhs) => format!("({op} {} {})", lhs.short_repr(), rhs.short_repr()),
			NodeValue::UnresolvedVariable(name, _) => format!("<var {name}>"),
			NodeValue::Print(expr, _) => format!("<print {}>", expr.short_repr()),
			NodeValue::Conditional(a, b, c) => {
				format!("<{} ? {} : {}>", a.short_repr(), b.short_repr(), c.short_repr())
			}
		}
	}
}

impl Display for NodeValue {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		let ctx = Context::get().format_without_span();
		ctx.is_used();

		match self {
			NodeValue::Token(token) => write!(f, "`{token}`"),
			NodeValue::Null => write!(f, "null"),
			NodeValue::Boolean(value) => write!(f, "{value}"),
			NodeValue::Unit => write!(f, "()"),
			NodeValue::Never => write!(f, "(!)"),
			NodeValue::Str(value) => write!(f, "{value:?}"),
			NodeValue::Int(value) => write!(f, "{value:?}"),
			NodeValue::Float(value) => write!(f, "{value:?}"),
			NodeValue::Variable(name, at, kind) => write!(f, "({name}{at:?} as {kind})"),
			NodeValue::Raw(nodes) => match nodes.len() {
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
			NodeValue::Sequence(nodes) => {
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
			NodeValue::Group(value) => write!(f, "{{ {value} }}"),
			NodeValue::Block(head, body) => {
				write!(f, "block({head}:")?;
				write!(f.indented(), "\n{body}")?;
				write!(f, "\n)")
			}
			NodeValue::If {
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
			NodeValue::For {
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
			NodeValue::Let(name, offset, expr) => {
				let offset = offset.value();
				write!(f, "let {name}{offset} = {expr}")
			}
			NodeValue::UnaryOp(op, _, arg) => write!(f, "({op} {arg})"),
			NodeValue::BinaryOp(op, _, lhs, rhs) => write!(f, "({lhs} {op} {rhs})"),
			NodeValue::UnresolvedVariable(var, offset) => {
				let offset = offset.value();
				write!(f, "{var}{offset}")
			}
			NodeValue::Print(expr, _) => write!(f, "print({expr})"),
			NodeValue::Conditional(cond, a, b) => write!(f, "{cond} ? {a} : {b}"),
		}
	}
}
