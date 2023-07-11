use super::*;

pub mod helper;
pub use helper::*;

pub mod eval_split_by;

pub use eval_split_by::*;

#[derive(Clone)]
pub struct NodeList {
	data: Arc<NodeListData>,
}

impl NodeList {
	pub fn from_single(scope: ScopeHandle, node: Node) -> Self {
		Self::new(scope, vec![node])
	}

	pub fn new(scope: ScopeHandle, nodes: Vec<Node>) -> Self {
		let data = NodeListData {
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
		Span::from_nodes(self.iter())
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

	pub fn get_next_evaluator(&self, max_precedence: Option<EvalPrecedence>) -> Result<Option<Evaluator>> {
		let evaluators = self.scope().get_evaluators().into_iter();
		let evaluators = evaluators.take_while(|x| {
			if let Some(max) = max_precedence {
				x.precedence() <= max
			} else {
				true
			}
		});

		let evaluators = evaluators.skip_while(|x| !x.can_apply(self));

		let mut evaluators = evaluators;
		if let Some(op) = evaluators.next() {
			let prec = op.precedence();
			let evaluators = evaluators.take_while(|x| x.precedence() == prec);
			let evaluators = evaluators.collect::<Vec<_>>();
			if evaluators.len() > 0 {
				let mut error =
					format!("ambiguous node list can accept multiple evaluators at the same precedence\n-> {op:?}");
				for eval in evaluators {
					let _ = write!(error, ", {eval:?}");
				}
				let _ = write!(error.indented(), "\n-> {self:?}");
				Err(Errors::from(error, self.span()))
			} else {
				Ok(Some(op))
			}
		} else {
			Ok(None)
		}
	}

	fn write<P: FnOnce(&mut Vec<Node>) -> bool>(&mut self, writer: P) {
		let mut nodes = self.data.nodes.write().unwrap();
		let nodes = Arc::make_mut(&mut nodes);
		if writer(nodes) {
			let mut version = self.data.version.write().unwrap();
			*version = *version + 1;
		}
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
