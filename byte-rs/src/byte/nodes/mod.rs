use super::*;

pub mod eval;
pub mod list;
pub mod operators;

pub use eval::*;
pub use list::*;
pub use operators::*;

const SHOW_INDENT: bool = false;

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
	Line(NodeList),
	Sequence(Vec<NodeList>),
	RawText(Span),
	Group(NodeList),
	Block(NodeList, NodeList),
	//----[ Logic ]-----------------------------------------------------------//
	If {
		expr: NodeList,
		if_true: NodeList,
		if_false: Option<NodeList>,
	},
	For {
		var: Symbol,
		offset: usize,
		from: NodeList,
		to: NodeList,
		body: NodeList,
	},
	//----[ AST ]-------------------------------------------------------------//
	Let(Symbol, Option<usize>, NodeList),
	UnaryOp(UnaryOp, NodeList),
	BinaryOp(BinaryOp, NodeList, NodeList),
	Variable(Symbol, Option<usize>),
	Print(NodeList, &'static str),
	Conditional(NodeList, NodeList, NodeList),
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

	pub fn children(&self) -> Vec<&NodeList> {
		match self {
			NodeValue::Token(_) => vec![],
			NodeValue::Null => vec![],
			NodeValue::Boolean(_) => vec![],
			NodeValue::Line(expr) => vec![expr],
			NodeValue::Sequence(ls) => ls.iter().map(|x| x).collect(),
			NodeValue::RawText(_) => vec![],
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

	pub fn get_dependencies<P: FnMut(&NodeList)>(&self, mut output: P) {
		for it in self.children() {
			output(it);
		}
	}
}

impl Display for NodeValue {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		match self {
			NodeValue::Line(nodes) => {
				write!(f, "Line::{nodes:?}")
			}
			_ => write!(f, "{self:?}"),
		}
	}
}

#[derive(Clone)]
pub struct Node {
	data: Arc<NodeData>,
}

struct NodeData {
	id: Id,
	span: Span,
	value: RwLock<NodeValue>,
	version: RwLock<usize>,
	scope: ScopeHandle,
}

impl Node {
	pub fn new(value: NodeValue, scope: ScopeHandle, (id, span): (Id, Span)) -> Self {
		let value = value.into();
		let version = 0.into();
		Self {
			data: NodeData {
				id,
				span,
				value,
				version,
				scope,
			}
			.into(),
		}
	}

	pub fn id(&self) -> Id {
		self.data.id.clone()
	}

	pub fn version(&self) -> usize {
		*self.data.version.read().unwrap()
	}

	pub fn val(&self) -> NodeValue {
		self.data.value.read().unwrap().clone()
	}

	pub fn span(&self) -> Span {
		self.data.span.clone()
	}

	pub fn offset(&self) -> usize {
		self.span().offset()
	}

	pub fn indent(&self) -> usize {
		self.span().indent()
	}

	pub fn scope(&self) -> Scope {
		self.data.scope.get()
	}

	pub fn scope_handle(&self) -> ScopeHandle {
		self.data.scope.clone()
	}

	pub fn get_dependencies<P: FnMut(&NodeList)>(&self, output: P) {
		self.val().get_dependencies(output)
	}

	//----------------------------------------------------------------------------------------------------------------//
	// Parsing helpers
	//----------------------------------------------------------------------------------------------------------------//

	pub fn is_symbol(&self, expected: &Symbol) -> bool {
		match self.val() {
			NodeValue::Token(Token::Symbol(symbol) | Token::Word(symbol)) => &symbol == expected,
			_ => false,
		}
	}

	pub fn is_keyword(&self, expected: &Symbol) -> bool {
		match self.val() {
			NodeValue::Token(Token::Word(symbol)) => &symbol == expected,
			_ => false,
		}
	}

	pub fn has_symbol(&self, symbol: &Symbol) -> bool {
		match self.val() {
			NodeValue::Token(Token::Symbol(s) | Token::Word(s)) => &s == symbol,
			_ => false,
		}
	}

	pub fn symbol(&self) -> Option<Symbol> {
		self.val().symbol()
	}
}

impl Debug for Node {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		write!(f, "{:?}", self.val())?;

		let ctx = Context::get();
		let format = ctx.format().with_mode(Mode::Minimal).with_separator(" @");
		ctx.with_format(format, || write!(f, "{}", self.span()))?;
		if SHOW_INDENT {
			write!(f, "~{}", self.indent())?;
		}
		Ok(())
	}
}

impl Display for Node {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		write!(f, "{}", self.val())?;

		let ctx = Context::get();
		let format = ctx.format().with_mode(Mode::Minimal).with_separator(" @");
		ctx.with_format(format, || write!(f, "{:#}", self.span()))?;
		if SHOW_INDENT {
			write!(f, "~{}", self.indent())?;
		}
		Ok(())
	}
}
