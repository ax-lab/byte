use std::collections::BTreeMap;

use super::*;

pub type BracketFn = Arc<dyn Fn(Symbol, NodeList, Symbol) -> Bit>;

#[derive(Clone, Default)]
pub struct BracketPairs {
	pairs: Arc<BTreeMap<Symbol, (Symbol, BracketFn)>>,
	reverse: Arc<HashSet<Symbol>>,
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

	pub fn add(&mut self, left: Symbol, right: Symbol, bracket_fn: BracketFn) {
		let pairs = Arc::make_mut(&mut self.pairs);
		pairs.insert(left, (right.clone(), bracket_fn));
		let reverse = Arc::make_mut(&mut self.reverse);
		reverse.insert(right);
	}

	fn parse_nodes(&self, nodes: &mut NodeList, new_lists: &mut Vec<NodeList>) -> Result<Vec<Node>> {
		let scope = nodes.scope();
		let mut items = nodes.as_vec_deque();
		self.parse_bracket(&scope, &mut items, None, new_lists).map(|x| x.0)
	}

	fn parse_bracket(
		&self,
		scope: &Scope,
		nodes: &mut VecDeque<Node>,
		pair: Option<(Span, Symbol, Symbol)>,
		new_lists: &mut Vec<NodeList>,
	) -> Result<(Vec<Node>, Span)> {
		let end = pair.as_ref().map(|(.., end)| end);
		let mut output = Vec::new();
		while let Some(next) = nodes.pop_front() {
			let pos = next.span().clone();
			if let Some(next_symbol) = next.symbol() {
				if Some(&next_symbol) == end {
					return Ok((output, pos));
				} else if let Some((sta, (end, bracket_fn))) = self.get_pair(&next) {
					let (list, end_pos) = self.parse_bracket(
						scope.clone(),
						nodes,
						Some((pos.clone(), sta.clone(), end.clone())),
						new_lists,
					)?;
					let list = NodeList::new(scope.handle(), list);
					new_lists.push(list.clone());
					let node = (bracket_fn)(sta, list, end);
					output.push(node.at(pos.to(end_pos)));
				} else if self.reverse.contains(&next_symbol) {
					let error = format!("unpaired right bracket `{next_symbol}`");
					let error = Errors::from(error, next.span().clone());
					return Err(error);
				} else {
					output.push(next);
				}
			} else {
				output.push(next);
			}
		}

		if let Some((pos, sta, end)) = pair {
			let error = format!("bracket `{sta}` is missing `{end}`");
			let error = Errors::from(error, pos);
			Err(error)
		} else {
			let span = output.last().map(|x| x.span()).unwrap_or_default();
			Ok((output, span))
		}
	}

	fn get_pair(&self, node: &Node) -> Option<(Symbol, (Symbol, BracketFn))> {
		match node.bit() {
			Bit::Token(Token::Symbol(left)) => self.pairs.get(left).map(|right| (left.clone(), right.clone())),
			_ => None,
		}
	}
}

impl IsEvaluator for BracketPairs {
	fn predicate(&self, node: &Node) -> bool {
		match node.bit() {
			Bit::Token(Token::Symbol(symbol)) => self.pairs.contains_key(symbol),
			_ => false,
		}
	}

	fn apply(&self, nodes: &mut NodeList, context: &mut EvalContext) -> Result<()> {
		let mut new_lists = Vec::new();
		match self.parse_nodes(nodes, &mut new_lists) {
			Ok(new_nodes) => {
				if new_lists.len() > 0 {
					nodes.replace_all(new_nodes);
					for it in new_lists {
						context.resolve_nodes(&it);
					}
				}
				Ok(())
			}
			Err(errors) => Err(errors),
		}
	}
}
