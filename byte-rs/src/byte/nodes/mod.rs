use super::*;

pub mod eval;
pub mod node_value;
pub mod operators;
pub mod parsing;

pub use eval::*;
pub use node_value::*;
pub use operators::*;
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
	parent: RwLock<Option<(Weak<NodeData>, usize)>>,
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
			parent: Default::default(),
		};
		let node = Self { data: data.into() };
		node.fixup_children(&node.data.value.read().unwrap());
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

	pub fn parent(&self) -> Option<Node> {
		self.get_parent().map(|x| x.0)
	}

	pub fn next(&self) -> Option<Node> {
		self.get_parent().and_then(|(node, index)| node.get(index + 1))
	}

	pub fn prev(&self) -> Option<Node> {
		self.get_parent()
			.and_then(|(node, index)| if index > 0 { node.get(index - 1) } else { None })
	}

	fn get_parent(&self) -> Option<(Node, usize)> {
		let parent = self.data.parent.read().unwrap();
		parent.as_ref().and_then(|(data, index)| {
			if let Some(data) = data.upgrade() {
				Some((Node { data }, *index))
			} else {
				None
			}
		})
	}

	fn set_parent(&self, new_parent: Option<(&Node, usize)>) {
		let new_parent = new_parent.map(|(node, index)| (Arc::downgrade(&node.data), index));
		self.write(|| {
			let mut parent = self.data.parent.write().unwrap();
			*parent = new_parent;
		});
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

			for it in value.children() {
				it.set_parent(None);
			}
			self.fixup_children(&new_value);

			*value = new_value;
			*span = new_span;
		});
	}

	fn write<T, P: FnOnce() -> T>(&self, write: P) -> T {
		let mut version = self.data.version.write().unwrap();
		*version = *version + 1;
		(write)()
	}

	fn fixup_children(&self, value: &NodeValue) {
		for (index, it) in value.children().iter().enumerate() {
			it.set_parent(Some((self, index)));
		}
	}

	//----------------------------------------------------------------------------------------------------------------//
	// Node value helpers
	//----------------------------------------------------------------------------------------------------------------//

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

	pub(crate) fn sanity_check(&self) {
		let mut prev = None;
		let mut next = self.get(0);
		for it in self.val().children() {
			assert_eq!(Some(it), next.as_ref());
			next = it.next();
			let parent = it.parent();
			if parent.as_ref().map(|x| x.id()) != Some(self.id()) {
				let msg = "\n\nsanity check failed: child/parent is broken:\n\n";
				panic!(
					"{msg}-> child:\n{it}\n\n-> should be parent:\n{self}\n\n-> but was: {}\n\n",
					if let Some(parent) = parent {
						format!("{parent}")
					} else {
						format!("(none)")
					}
				);
			}
			assert!(it.prev() == prev);
			prev = Some(it.clone());
			it.sanity_check();
		}
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
