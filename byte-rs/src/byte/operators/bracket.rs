use std::collections::BTreeMap;

use super::*;

pub type BracketFn = Arc<dyn Fn(Name, NodeList, Name) -> Node>;

#[derive(Clone, Default)]
pub struct BracketPairs {
	pairs: Arc<BTreeMap<Name, (Name, BracketFn)>>,
	reverse: Arc<HashSet<Name>>,
}

impl Hash for BracketPairs {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		for (k, v) in self.pairs.iter() {
			k.hash(state);
			v.0.hash(state);
		}
	}
}

impl Debug for BracketPairs {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "BracketPairs<")?;
		for (i, it) in self.pairs.iter().enumerate() {
			if i > 0 {
				write!(f, ", ")?;
			}
			let sta = &it.0;
			let (end, ..) = &it.1;
			write!(f, "`{sta} {end}`")?;
		}
		write!(f, ">")
	}
}

impl PartialEq for BracketPairs {
	fn eq(&self, other: &Self) -> bool {
		if self.pairs.len() == other.pairs.len() {
			for (key, val) in self.pairs.iter() {
				if let Some((sta, ..)) = other.pairs.get(key) {
					if sta != &val.0 {
						return false;
					}
				} else {
					return false;
				}
			}
			true
		} else {
			false
		}
	}
}

impl Eq for BracketPairs {}

impl BracketPairs {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn add(&mut self, left: Name, right: Name, bracket_fn: BracketFn) {
		let pairs = Arc::make_mut(&mut self.pairs);
		pairs.insert(left, (right.clone(), bracket_fn));
		let reverse = Arc::make_mut(&mut self.reverse);
		reverse.insert(right);
	}

	fn parse_nodes(&self, nodes: &NodeList, new_lists: &mut Vec<NodeList>) -> Result<Vec<NodeData>> {
		let mut items = nodes.as_vec_deque();
		self.parse_bracket(nodes.scope().handle(), &mut items, None, new_lists)
	}

	fn parse_bracket(
		&self,
		scope: Handle<Scope>,
		nodes: &mut VecDeque<NodeData>,
		pair: Option<(Span, Name, Name)>,
		new_lists: &mut Vec<NodeList>,
	) -> Result<Vec<NodeData>> {
		let end = pair.as_ref().map(|(.., end)| end);
		let mut output = Vec::new();
		while let Some(node) = nodes.pop_front() {
			if let Some(name) = node.name() {
				if Some(&name) == end {
					return Ok(output);
				} else if let Some((sta, (end, bracket_fn))) = self.get_pair(&node) {
					let span = node.span().clone();
					let list = self.parse_bracket(
						scope.clone(),
						nodes,
						Some((span.clone(), sta.clone(), end.clone())),
						new_lists,
					)?;
					let list = NodeList::new(scope.clone(), list);
					new_lists.push(list.clone());
					let node = (bracket_fn)(sta, list, end);
					output.push(node.at(span));
				} else if self.reverse.contains(&name) {
					let error = format!("unpaired end bracket `{name}`");
					let error = Errors::from_at(error, node.span().clone());
					return Err(error);
				} else {
					output.push(node);
				}
			} else {
				output.push(node);
			}
		}

		if let Some((pos, sta, end)) = pair {
			let error = format!("bracket `{sta}` is missing `{end}`");
			let error = Errors::from_at(error, pos);
			Err(error)
		} else {
			Ok(output)
		}
	}

	fn get_pair(&self, node: &NodeData) -> Option<(Name, (Name, BracketFn))> {
		match node.get() {
			Node::Symbol(name) => self.pairs.get(name).map(|end| (name.clone(), end.clone())),
			_ => None,
		}
	}
}

impl IsOperator for BracketPairs {
	fn precedence(&self) -> Precedence {
		Precedence::Brackets
	}

	fn predicate(&self, node: &Node) -> bool {
		match node {
			Node::Symbol(name) => self.pairs.contains_key(name),
			_ => false,
		}
	}

	fn apply(&self, context: &mut OperatorContext, errors: &mut Errors) {
		let nodes = context.nodes();
		let mut new_lists = Vec::new();
		match self.parse_nodes(nodes, &mut new_lists) {
			Ok(new_nodes) => {
				if new_lists.len() > 0 {
					nodes.replace_all(new_nodes);
					for it in new_lists {
						context.resolve_nodes(&it);
					}
				}
			}
			Err(errs) => errors.append(&errs),
		}
	}
}
