use super::*;

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

	//----------------------------------------------------------------------------------------------------------------//
	// Operators
	//----------------------------------------------------------------------------------------------------------------//

	pub fn iter(&self) -> NodeListIterator {
		let nodes = self.data.nodes.read().unwrap();
		let nodes = nodes.clone();
		NodeListIterator { index: 0, nodes }
	}

	pub fn contains<P: Fn(&Node) -> bool>(&self, predicate: P) -> bool {
		let nodes = self.data.nodes.read().unwrap();
		nodes.iter().any(|x| predicate(x))
	}

	pub fn contains_delimiter_pair(&self, sta: &Symbol, end: &Symbol) -> bool {
		let nodes = self.data.nodes.read().unwrap();

		let nodes = nodes.iter();
		let mut nodes = nodes.skip_while(|x| x.symbol().as_ref() != Some(sta));
		if let Some(..) = nodes.next() {
			let mut nodes = nodes.skip_while(|x| x.symbol().as_ref() != Some(end));
			nodes.next().is_some()
		} else {
			false
		}
	}

	pub fn split_ternary(&self, sta: &Symbol, end: &Symbol) -> Option<(Vec<Node>, Vec<Node>, Vec<Node>)> {
		let nodes = self.data.nodes.read().unwrap();
		for i in (0..nodes.len()).rev() {
			if nodes[i].has_symbol(sta) {
				for j in i + 1..nodes.len() {
					if nodes[j].has_symbol(end) {
						let a = nodes[0..i].to_vec();
						let b = nodes[i + 1..j].to_vec();
						let c = nodes[j + 1..].to_vec();
						return Some((a, b, c));
					}
				}
			}
		}
		None
	}

	pub fn get_next_operator(&self, max_precedence: Option<Precedence>) -> Result<Option<Operator>> {
		let operators = self.scope().get_operators().into_iter();
		let operators = operators.take_while(|x| {
			if let Some(max) = max_precedence {
				x.precedence() <= max
			} else {
				true
			}
		});

		let operators = operators.skip_while(|x| !x.can_apply(self));

		let mut operators = operators;
		if let Some(op) = operators.next() {
			let prec = op.precedence();
			let operators = operators.take_while(|x| x.precedence() == prec);
			let operators = operators.collect::<Vec<_>>();
			if operators.len() > 0 {
				let mut error =
					format!("ambiguous node list can accept multiple operators at the same precedence\n-> {op:?}");
				for op in operators {
					let _ = write!(error, ", {op:?}");
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

	pub fn map_nodes<P: FnMut(&Node) -> Option<Vec<Node>>>(&mut self, mut predicate: P) {
		let mut changed = false;
		{
			let mut nodes = self.data.nodes.write().unwrap();
			let nodes = Arc::make_mut(&mut nodes);

			*nodes = std::mem::take(nodes)
				.into_iter()
				.flat_map(|it| {
					if let Some(nodes) = predicate(&it) {
						changed = true;
						nodes
					} else {
						vec![it]
					}
				})
				.collect();
		}
		if changed {
			self.inc_version()
		}
	}

	pub fn replace<P: FnMut(&Node) -> Option<Node>>(&mut self, mut replace: P) {
		let changed = {
			let mut nodes = self.data.nodes.write().unwrap();
			let nodes = Arc::make_mut(&mut nodes);
			let mut changed = false;
			for it in nodes.iter_mut() {
				if let Some(new_node) = replace(it) {
					*it = new_node;
					changed = true;
				}
			}
			changed
		};

		if changed {
			self.inc_version();
		}
	}

	pub fn replace_all(&mut self, new_nodes: Vec<Node>) {
		{
			let mut nodes = self.data.nodes.write().unwrap();
			let nodes = Arc::make_mut(&mut nodes);
			*nodes = new_nodes;
		}
		self.inc_version();
	}

	pub fn split_by<P: FnMut(&Node) -> bool, S: FnMut(NodeList) -> Node>(&mut self, mut split: P, mut node: S) {
		let changed = {
			let mut new_nodes = Vec::new();
			let mut line = Vec::new();

			let mut nodes = self.data.nodes.write().unwrap();
			let nodes = Arc::make_mut(&mut nodes);

			for it in nodes.iter() {
				if split(it) {
					let nodes = Self::new(self.data.scope.clone(), std::mem::take(&mut line));
					new_nodes.push(node(nodes));
				} else {
					line.push(it.clone());
				}
			}

			if line.len() > 0 {
				let nodes = Self::new(self.data.scope.clone(), std::mem::take(&mut line));
				new_nodes.push(node(nodes));
			}

			if new_nodes.len() > 0 {
				*nodes = new_nodes;
				true
			} else {
				false
			}
		};

		if changed {
			self.inc_version();
		}
	}

	pub fn split_by_items<P: FnMut(&Node) -> bool>(&mut self, mut split: P) -> Vec<NodeList> {
		let mut new_nodes = Vec::new();
		let mut line = Vec::new();

		let scope = &self.data.scope;

		let mut nodes = self.data.nodes.write().unwrap();
		let nodes = Arc::make_mut(&mut nodes);

		for it in nodes.iter() {
			if split(it) {
				let nodes = std::mem::take(&mut line);
				new_nodes.push(NodeList::new(scope.clone(), nodes));
			} else {
				line.push(it.clone());
			}
		}

		if line.len() > 0 {
			let nodes = Self::new(self.data.scope.clone(), std::mem::take(&mut line));
			new_nodes.push(nodes);
		}
		new_nodes
	}

	// TODO: move the parsing functions to the eval context and allow them to receive a context reference

	pub fn fold_first<P: FnMut(&Node) -> bool, S: FnMut(NodeList, Node, NodeList) -> Node>(
		&mut self,
		mut fold: P,
		mut make_node: S,
	) {
		let mut changed = false;
		{
			let mut nodes = self.data.nodes.write().unwrap();
			for i in 0..nodes.len() {
				if fold(&nodes[i]) {
					let scope = &self.data.scope;
					let nodes = Arc::make_mut(&mut nodes);

					let lhs = nodes[0..i].to_vec();
					let cur = nodes[i].clone();
					let rhs = nodes[i + 1..].to_vec();
					let lhs = NodeList::new(scope.clone(), lhs);
					let rhs = NodeList::new(scope.clone(), rhs);
					let node = make_node(lhs, cur, rhs);
					*nodes = vec![node];
					changed = true;
					break;
				}
			}
		}

		if changed {
			self.inc_version();
		}
	}

	pub fn fold_last<P: FnMut(&Node) -> bool, S: FnMut(NodeList, Node, NodeList) -> Node>(
		&mut self,
		mut fold: P,
		mut make_node: S,
	) {
		let mut changed = false;
		{
			let mut nodes = self.data.nodes.write().unwrap();
			for i in (0..nodes.len()).rev() {
				if fold(&nodes[i]) {
					let scope = &self.data.scope;
					let nodes = Arc::make_mut(&mut nodes);

					let lhs = nodes[0..i].to_vec();
					let cur = nodes[i].clone();
					let rhs = nodes[i + 1..].to_vec();
					let lhs = NodeList::new(scope.clone(), lhs);
					let rhs = NodeList::new(scope.clone(), rhs);
					let node = make_node(lhs, cur, rhs);
					*nodes = vec![node];
					changed = true;
					break;
				}
			}
		}

		if changed {
			self.inc_version();
		}
	}

	fn inc_version(&mut self) {
		let mut version = self.data.version.write().unwrap();
		*version = *version + 1;
	}

	//----------------------------------------------------------------------------------------------------------------//
	// Parsing helpers
	//----------------------------------------------------------------------------------------------------------------//

	pub fn get(&self, index: usize) -> Option<Node> {
		let nodes = self.data.nodes.read().unwrap();
		nodes.get(index).cloned()
	}

	pub fn get_symbol(&self, index: usize) -> Option<Symbol> {
		let nodes = self.data.nodes.read().unwrap();
		nodes.get(index).and_then(|x| x.symbol())
	}

	pub fn test_at<P: FnOnce(&Node) -> bool>(&self, index: usize, predicate: P) -> bool {
		let nodes = self.data.nodes.read().unwrap();
		nodes.get(index).map(|x| predicate(x)).unwrap_or(false)
	}

	pub fn is_identifier(&self, index: usize) -> bool {
		self.test_at(index, |x| matches!(x.bit(), Bit::Token(Token::Word(..))))
	}

	pub fn is_keyword(&self, index: usize, word: &Symbol) -> bool {
		self.test_at(index, |x| x.is_word(word))
	}

	pub fn is_symbol(&self, index: usize, symbol: &Symbol) -> bool {
		self.test_at(index, |x| x.is_symbol(symbol))
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
