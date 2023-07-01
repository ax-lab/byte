use super::*;

#[derive(Clone)]
pub struct NodeList {
	data: Arc<NodeListData>,
}

has_traits!(NodeList: WithRepr);

impl NodeList {
	pub fn from_single(scope: Handle<Scope>, node: NodeData) -> Self {
		Self::new(scope, vec![node])
	}

	pub fn new(scope: Handle<Scope>, nodes: Vec<NodeData>) -> Self {
		let data = NodeListData {
			version: RwLock::new(0),
			scope,
			nodes: RwLock::new(Arc::new(nodes)),
		};
		Self { data: data.into() }
	}

	pub fn version(&self) -> usize {
		*self.data.version.read().unwrap()
	}

	pub fn scope(&self) -> HandleRef<Scope> {
		self.data.scope.get()
	}

	pub fn scope_mut(&mut self) -> HandleMut<Scope> {
		unsafe { self.data.scope.get_mut() }
	}

	pub fn span(&self) -> Span {
		let nodes = self.data.nodes.read().unwrap();
		nodes.first().map(|x| x.span().clone()).unwrap_or(Span::None)
	}

	pub fn offset(&self) -> usize {
		let nodes = self.data.nodes.read().unwrap();
		nodes.first().map(|x| x.offset()).unwrap_or(0)
	}

	pub fn len(&self) -> usize {
		let nodes = self.data.nodes.read().unwrap();
		nodes.len()
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
		nodes.iter().any(|x| predicate(x.get()))
	}

	pub fn get_next_operator(&self, max_precedence: Option<Precedence>) -> Result<Option<Operator>> {
		let operators = self.data.scope.read(|x| x.get_operators()).into_iter();
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
				Err(Errors::from_at(error, self.span()))
			} else {
				Ok(Some(op))
			}
		} else {
			Ok(None)
		}
	}

	pub fn map_nodes<P: FnMut(&NodeData) -> Option<Vec<NodeData>>>(&mut self, mut predicate: P) {
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

	pub fn replace<P: FnMut(&NodeData) -> Option<NodeData>>(&mut self, mut replace: P) {
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

	pub fn split_by<P: FnMut(&NodeData) -> bool, S: FnMut(NodeList) -> NodeData>(&mut self, mut split: P, mut node: S) {
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

	// TODO: move the parsing functions to the eval context and allow them to receive a context reference

	pub fn fold_first<P: FnMut(&NodeData) -> bool, S: FnMut(NodeList, NodeData, NodeList) -> NodeData>(
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

	pub fn fold_last<P: FnMut(&NodeData) -> bool, S: FnMut(NodeList, NodeData, NodeList) -> NodeData>(
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

	pub fn get(&self, index: usize) -> Option<NodeData> {
		let nodes = self.data.nodes.read().unwrap();
		nodes.get(index).cloned()
	}

	pub fn get_name(&self, index: usize) -> Option<Name> {
		let nodes = self.data.nodes.read().unwrap();
		nodes.get(index).and_then(|x| x.name())
	}

	pub fn test_at<P: FnOnce(&NodeData) -> bool>(&self, index: usize, predicate: P) -> bool {
		let nodes = self.data.nodes.read().unwrap();
		nodes.get(index).map(|x| predicate(x)).unwrap_or(false)
	}

	pub fn is_identifier(&self, index: usize) -> bool {
		self.test_at(index, |x| matches!(x.get(), Node::Word(..)))
	}

	pub fn is_keyword(&self, index: usize, word: &str) -> bool {
		self.test_at(index, |x| x.is_word(word))
	}

	pub fn is_symbol(&self, index: usize, symbol: &str) -> bool {
		self.test_at(index, |x| x.is_symbol(symbol))
	}
}

impl WithRepr for NodeList {
	fn output(&self, mode: ReprMode, format: ReprFormat, output: &mut dyn std::fmt::Write) -> std::fmt::Result {
		let nodes = self.data.nodes.read().unwrap();
		let _ = (mode, format);
		if format == ReprFormat::Full {
			write!(output, "Nodes(")?;
			for (n, it) in nodes.iter().enumerate() {
				let mut output = IndentedFormatter::new(output);
				write!(output, "\n[{n}] = ")?;
				write!(output, "{it}")?;
				if let Some(location) = it.span().location(0) {
					write!(output, "\t # at {location}")?;
				}
			}
			if nodes.len() > 0 {
				write!(output, "\n")?;
			}
			write!(output, ")")
		} else {
			write!(output, "{{")?;
			for (n, it) in nodes.iter().enumerate() {
				let mut output = IndentedFormatter::new(output);
				write!(output, "{}", if n > 0 { ", " } else { " " })?;
				write!(output, "{it}")?;
			}
			if nodes.len() > 0 {
				write!(output, " ")?;
			}
			write!(output, "}}")
		}
	}
}

fmt_from_repr!(NodeList);

impl PartialEq for NodeList {
	fn eq(&self, other: &Self) -> bool {
		Arc::as_ptr(&self.data) == Arc::as_ptr(&other.data)
	}
}

impl Eq for NodeList {}

//====================================================================================================================//
// NodeListData
//====================================================================================================================//

struct NodeListData {
	version: RwLock<usize>,
	scope: Handle<Scope>,
	nodes: RwLock<Arc<Vec<NodeData>>>,
}

//====================================================================================================================//
// Iterator
//====================================================================================================================//

pub struct NodeListIterator {
	index: usize,
	nodes: Arc<Vec<NodeData>>,
}

impl Iterator for NodeListIterator {
	type Item = NodeData;

	fn next(&mut self) -> Option<Self::Item> {
		let output = self.nodes.get(self.index);
		if output.is_some() {
			self.index += 1;
		}
		output.cloned()
	}
}
