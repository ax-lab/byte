use super::*;

/// Enumeration of all available language elements.
///
/// Nodes relate to the source code, representing language constructs of all
/// levels, from files, raw text, and tokens, all the way to fully fledged
/// definitions.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum NodeValue {
	Token(Token),
	Null,
	Boolean(bool),
	//----[ Structural ]------------------------------------------------------//
	Raw(Arc<Vec<Node>>),
	Line(Node),
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
		offset: usize,
		from: Node,
		to: Node,
		body: Node,
	},
	//----[ AST ]-------------------------------------------------------------//
	Let(Symbol, Option<usize>, Node),
	UnaryOp(UnaryOp, Node),
	BinaryOp(BinaryOp, Node, Node),
	Variable(Symbol, Option<usize>),
	Print(Node, &'static str),
	Conditional(Node, Node, Node),
}

impl NodeValue {
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
			NodeValue::Raw(ls) => ls.iter().map(|x| x).collect(),
			NodeValue::Line(expr) => vec![expr],
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
			NodeValue::UnaryOp(_, expr) => vec![expr],
			NodeValue::BinaryOp(_, lhs, rhs) => vec![lhs, rhs],
			NodeValue::Variable(..) => vec![],
			NodeValue::Print(expr, _) => vec![expr],
			NodeValue::Conditional(cond, t, f) => vec![cond, t, f],
		}
	}

	pub fn get_dependencies<P: FnMut(&Node)>(&self, mut output: P) {
		for it in self.children() {
			output(it);
		}
	}
}

impl Display for NodeValue {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		let ctx = Context::get().format_without_span();
		ctx.is_used();

		match self {
			NodeValue::Token(token) => write!(f, "{token}"),
			NodeValue::Null => write!(f, "(null)"),
			NodeValue::Boolean(value) => write!(f, "({value})"),
			NodeValue::Raw(nodes) => match nodes.len() {
				0 => write!(f, "<>"),
				1 => {
					let output = format!("{}", nodes[0]);
					if output.contains('\n') || output.len() > 50 {
						write!(f.indented(), "<\n{output}")?;
						write!(f, "\n>")
					} else {
						write!(f, "<{output}>")
					}
				}
				_ => {
					let ctx = ctx.format_with_span();
					ctx.is_used();
					write!(f, "<# Raw:")?;
					let mut out = f.indented();
					for (n, it) in nodes.iter().enumerate() {
						write!(out, "\n# {n}:")?;
						write!(out.indented(), "\n{it}")?;
					}
					write!(f, "\n>")
				}
			},
			NodeValue::Line(value) => {
				let ctx = ctx.format_with_span();
				ctx.is_used();
				write!(f.indented(), "# Line:\n{value}")
			}
			NodeValue::Sequence(nodes) => {
				let ctx = ctx.format_with_span();
				ctx.is_used();
				write!(f, "# Sequence:")?;
				let mut out = f.indented();
				for (n, it) in nodes.iter().enumerate() {
					write!(out, "\n# {n}:")?;
					write!(out.indented(), "\n{it}")?;
				}
				Ok(())
			}
			NodeValue::Group(value) => write!(f, "Group({value})"),
			NodeValue::Block(head, body) => {
				write!(f, "Block({head}:")?;
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
				write!(f, "for {var}#{offset} in {from}..{to} {{")?;
				write!(f.indented(), "\n{body}")?;
				write!(f, "\n}}")
			}
			NodeValue::Let(name, offset, expr) => {
				let offset = offset.unwrap_or(0);
				write!(f, "let {name}_{offset} = {expr}")
			}
			NodeValue::UnaryOp(op, arg) => write!(f, "{op} {arg}"),
			NodeValue::BinaryOp(op, lhs, rhs) => write!(f, "({lhs} {op} {rhs})"),
			NodeValue::Variable(var, offset) => {
				let offset = offset.unwrap_or(0);
				write!(f, "{var}_{offset}")
			}
			NodeValue::Print(expr, _) => write!(f, "Print({expr})"),
			NodeValue::Conditional(cond, a, b) => write!(f, "{cond} ? {a} : {b}"),
		}
	}
}
