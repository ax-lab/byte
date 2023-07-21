use super::*;

pub mod eval;
pub mod node_value;
pub mod parsing;

pub use eval::*;
pub use node_value::*;
pub use parsing::*;

const SHOW_INDENT: bool = false;

#[derive(Clone)]
pub struct Node {
	data: Arc<NodeData>,
}

struct NodeData {
	id: Id,
	span: RwLock<Span>,
	value: RwLock<NodeValue>,
	version: RwLock<usize>,
	scope: ScopeHandle,
}

impl Node {
	pub fn new(value: NodeValue, scope: ScopeHandle, span: Span) -> Self {
		let value = value.into();
		let version = 0.into();
		let id = id();
		let span = span.into();
		let data = NodeData {
			id,
			span,
			value,
			version,
			scope,
		};
		let node = Self { data: data.into() };
		node
	}

	pub fn raw(nodes: Vec<Node>, scope: ScopeHandle) -> Self {
		let span = Span::from_node_vec(&nodes);
		NodeValue::Raw(nodes.into()).at(scope, span)
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
		self.data.span.read().unwrap().clone()
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

	pub fn get_dependencies<P: FnMut(&Node)>(&self, output: P) {
		self.val().get_dependencies(output)
	}

	pub fn set_value(&mut self, new_value: NodeValue, new_span: Span) {
		self.write(|| {
			let mut value = self.data.value.write().unwrap();
			let mut span = self.data.span.write().unwrap();
			*value = new_value;
			*span = new_span;
		});
	}

	pub unsafe fn set_value_inner(&self, new_value: NodeValue, new_span: Span) {
		self.write(|| {
			let mut value = self.data.value.write().unwrap();
			let mut span = self.data.span.write().unwrap();
			*value = new_value;
			*span = new_span;
		});
	}

	fn write<T, P: FnOnce() -> T>(&self, write: P) -> T {
		let mut version = self.data.version.write().unwrap();
		*version = *version + 1;
		(write)()
	}

	//----------------------------------------------------------------------------------------------------------------//
	// Node value helpers
	//----------------------------------------------------------------------------------------------------------------//

	pub fn short_repr(&self) -> String {
		self.val().short_repr()
	}

	/// Number of child nodes.
	pub fn len(&self) -> usize {
		self.val().len()
	}

	/// Get a node children by its index.
	pub fn get(&self, index: usize) -> Option<Node> {
		self.val().get(index).cloned()
	}

	/// Return a new [`NodeValue::Raw`] from a slice of this node's children.
	pub fn slice<T: RangeBounds<usize>>(&self, range: T) -> Node {
		let scope = self.scope_handle();
		let node = self.val();

		// TODO: maybe have a `can_slice` property
		assert!(matches!(node, NodeValue::Raw(..))); // we don't want slice to be used with any node
		let list = node.children();
		let range = compute_range(range, list.len());
		let index = range.start;
		let slice = &list[range];
		let span = Span::from_nodes(slice);
		let span = span.or_with(|| list.get(index).map(|x| x.span().pos()).unwrap_or_default());
		let list = Arc::new(slice.iter().map(|x| (*x).clone()).collect());
		NodeValue::Raw(list).at(scope, span)
	}

	/// Iterator over this node's children.
	pub fn iter(&self) -> impl Iterator<Item = Node> {
		let node = self.val();
		let list = node.iter().cloned().collect::<Vec<_>>();
		list.into_iter()
	}

	//----------------------------------------------------------------------------------------------------------------//
	// Parsing helpers
	//----------------------------------------------------------------------------------------------------------------//

	pub fn to_raw(self) -> Node {
		let span = self.span();
		let scope = self.scope_handle();
		NodeValue::Raw(vec![self].into()).at(scope, span)
	}

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

impl Hash for Node {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		let value = self.data.value.read().unwrap();
		value.hash(state)
	}
}

impl PartialEq for Node {
	fn eq(&self, other: &Self) -> bool {
		if Arc::as_ptr(&self.data) == Arc::as_ptr(&other.data) {
			true
		} else {
			let va = self.data.value.read().unwrap();
			let vb = other.data.value.read().unwrap();
			*va == *vb && self.data.scope == other.data.scope
		}
	}
}

impl Eq for Node {}

impl Debug for Node {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		write!(f, "{}", self.val())?;

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
