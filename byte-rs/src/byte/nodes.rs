use super::*;

/// Enumeration of all available language nodes.
///
/// Nodes relate to the source code, representing language constructs of all
/// levels, from files, raw text, and tokens, all the way to fully fledged
/// definitions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Node {
	Break,
	Indent(usize),
	Comment,
	Word(Name),
	Symbol(Name),
	Literal(String),
	Integer(u128),
	RawText(Span),
	Line(Vec<Node>),
	File(Span),
	Module(Span),
	Import(String),
}

impl Node {
	pub fn at(self, span: Span) -> NodeData {
		NodeData::new(self, span)
	}
}

/// Encapsulates a [`Node`] with additional data.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NodeData {
	id: Id,
	node: Node,
	span: Span,
}

impl NodeData {
	pub fn new(node: Node, span: Span) -> Self {
		let id = new_id();
		Self { id, node, span }
	}

	pub fn id(&self) -> Id {
		self.id
	}

	pub fn get(&self) -> &Node {
		&self.node
	}

	pub fn to_inner(self) -> Node {
		self.node
	}

	pub fn span(&self) -> &Span {
		&self.span
	}
}

#[derive(Clone)]
pub struct NodeList {
	data: Arc<NodeListData>,
}

has_traits!(NodeList: WithRepr);

impl NodeList {
	pub fn from_single(scope: Handle<Scope>, node: NodeData) -> Self {
		let data = NodeListData {
			scope,
			nodes: vec![node],
		};
		Self { data: data.into() }
	}

	pub fn span(&self) -> Span {
		self.data.nodes.first().map(|x| x.span().clone()).unwrap_or(Span::None)
	}

	pub fn len(&self) -> usize {
		self.data.nodes.len()
	}

	pub fn contains<P: Fn(&Node) -> bool>(&self, predicate: P) -> bool {
		self.data.nodes.iter().any(|x| predicate(x.get()))
	}

	pub fn nodes(&self) -> &[NodeData] {
		&self.data.nodes
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
}

struct NodeListData {
	scope: Handle<Scope>,
	nodes: Vec<NodeData>,
}

impl WithRepr for NodeList {
	fn output(&self, mode: ReprMode, format: ReprFormat, output: &mut dyn std::fmt::Write) -> std::fmt::Result {
		let _ = (mode, format);
		write!(output, "Nodes(")?;
		for it in self.nodes().iter() {
			let mut output = IndentedFormatter::new(output);
			write!(output, "\n{it:?}")?;
		}
		if self.nodes().len() > 0 {
			write!(output, "\n")?;
		}
		write!(output, ")")
	}
}

fmt_from_repr!(NodeList);
