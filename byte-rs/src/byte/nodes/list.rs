use super::*;

pub mod helper;
pub use helper::*;

pub mod parse_brackets;
pub mod parse_expr;
pub mod parse_filter;
pub mod parse_fold;
pub mod parse_keyword;
pub mod parse_replace;
pub mod parse_split;
pub mod parse_ternary;

pub use parse_brackets::*;
pub use parse_expr::*;
pub use parse_filter::*;
pub use parse_fold::*;
pub use parse_keyword::*;
pub use parse_replace::*;
pub use parse_split::*;
pub use parse_ternary::*;

#[derive(Clone)]
pub struct NodeList {
	data: Arc<NodeListData>,
}

impl NodeList {
	pub fn from_single(scope: ScopeHandle, node: Node) -> Self {
		Self::new(scope, vec![node])
	}

	pub fn new(scope: ScopeHandle, nodes: Vec<Node>) -> Self {
		let span = Span::from_nodes(nodes.iter().cloned());
		let data = NodeListData {
			span: RwLock::new(span),
			version: RwLock::new(0),
			scope,
			nodes: RwLock::new(Arc::new(nodes)),
		};
		Self { data: data.into() }
	}

	pub fn as_vec_deque(&self) -> VecDeque<Node> {
		VecDeque::from_iter(self.iter())
	}

	pub fn as_vec(&self) -> Vec<Node> {
		Vec::from_iter(self.iter())
	}

	pub fn is_same(&self, other: &NodeList) -> bool {
		Arc::as_ptr(&self.data) == Arc::as_ptr(&other.data)
	}

	pub fn version(&self) -> usize {
		*self.data.version.read().unwrap()
	}

	pub fn scope(&self) -> Scope {
		self.data.scope.get()
	}

	pub fn scope_handle(&self) -> ScopeHandle {
		self.data.scope.clone()
	}

	pub fn span(&self) -> Span {
		self.data.span.read().unwrap().clone()
	}

	pub fn offset(&self) -> usize {
		let nodes = self.data.nodes.read().unwrap();
		nodes.first().map(|x| x.offset()).unwrap_or(0)
	}

	pub fn len(&self) -> usize {
		let nodes = self.data.nodes.read().unwrap();
		nodes.len()
	}

	pub fn slice<T: RangeBounds<usize>>(&self, range: T) -> NodeList {
		let nodes = self.data.nodes.read().unwrap();
		let range = compute_range(range, self.len());
		Self::new(self.data.scope.clone(), nodes[range].to_vec())
	}

	pub fn iter(&self) -> NodeListIterator {
		let nodes = self.data.nodes.read().unwrap();
		let nodes = nodes.clone();
		NodeListIterator { index: 0, nodes }
	}

	pub fn contains<P: Fn(&Node) -> bool>(&self, predicate: P) -> bool {
		let nodes = self.data.nodes.read().unwrap();
		nodes.iter().any(|x| predicate(x))
	}

	pub fn get(&self, index: usize) -> Option<Node> {
		let nodes = self.data.nodes.read().unwrap();
		nodes.get(index).cloned()
	}

	pub fn write<P: FnOnce(&mut Vec<Node>) -> bool>(&mut self, writer: P) {
		let mut nodes = self.data.nodes.write().unwrap();
		let nodes = Arc::make_mut(&mut nodes);
		if writer(nodes) {
			let mut version = self.data.version.write().unwrap();
			*version = *version + 1;

			let mut span = self.data.span.write().unwrap();
			*span = Span::from_nodes(nodes.iter().cloned());
		}
	}

	pub fn write_res<P: FnOnce(&mut Vec<Node>) -> Result<bool>>(&mut self, writer: P) -> Result<()> {
		let mut nodes = self.data.nodes.write().unwrap();
		let nodes = Arc::make_mut(&mut nodes);
		if writer(nodes)? {
			let mut version = self.data.version.write().unwrap();
			*version = *version + 1;
		}
		Ok(())
	}
}

impl Debug for NodeList {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		let ctx = Context::get();
		let nodes = self.data.nodes.read().unwrap();
		write!(f, "Nodes(")?;
		for (n, it) in nodes.iter().enumerate() {
			let mut output = IndentedFormatter::new(f);
			if n == 0 && !ctx.format().nested() {
				if let Some(location) = self.span().location() {
					write!(output, "\n# {location}")?;
				}
			}

			ctx.with_format(ctx.format().as_nested(), || {
				write!(output, "\n")?;
				if nodes.len() > 1 {
					write!(output, "[{n}] = ")?;
				}
				write!(output, "{it}")
			})?
		}
		if nodes.len() > 0 {
			write!(f, "\n")?;
		}
		write!(f, ")")
	}
}

impl Display for NodeList {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		let nodes = self.data.nodes.read().unwrap();
		write!(f, "{{")?;
		for (n, it) in nodes.iter().enumerate() {
			let mut output = IndentedFormatter::new(f);
			write!(output, "{}", if n > 0 { ", " } else { " " })?;
			write!(output, "{it}")?;
		}
		if nodes.len() > 0 {
			write!(f, " ")?;
		}
		write!(f, "}}")
	}
}

impl PartialEq for NodeList {
	fn eq(&self, other: &Self) -> bool {
		Arc::as_ptr(&self.data) == Arc::as_ptr(&other.data)
	}
}

impl Eq for NodeList {}

impl Hash for NodeList {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		let nodes = self.data.nodes.read().unwrap();
		for it in nodes.iter() {
			it.id().hash(state);
		}
	}
}

//====================================================================================================================//
// NodeListData
//====================================================================================================================//

struct NodeListData {
	span: RwLock<Span>,
	version: RwLock<usize>,
	scope: ScopeHandle,
	nodes: RwLock<Arc<Vec<Node>>>,
}

//====================================================================================================================//
// Iterator
//====================================================================================================================//

pub struct NodeListIterator {
	index: usize,
	nodes: Arc<Vec<Node>>,
}

impl Iterator for NodeListIterator {
	type Item = Node;

	fn next(&mut self) -> Option<Self::Item> {
		let output = self.nodes.get(self.index);
		if output.is_some() {
			self.index += 1;
		}
		output.cloned()
	}
}
