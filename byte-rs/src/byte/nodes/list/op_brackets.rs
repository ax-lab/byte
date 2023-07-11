use std::collections::BTreeMap;

use super::*;

pub type BracketFn = Arc<dyn Fn(Symbol, NodeList, Symbol) -> Bit>;

/// Configures a set of bracket pairs and provides parsing for [`NodeList`].
#[derive(Clone, Default)]
pub struct BracketPairs {
	pairs: Arc<BTreeMap<Symbol, (Symbol, BracketFn)>>,
	reverse: Arc<HashSet<Symbol>>,
}

impl NodeBracketParser for BracketPairs {
	type Bracket = SymbolBracket;

	fn is_bracket(&self, node: &Node) -> bool {
		if let Some(symbol) = node.symbol() {
			self.pairs.get(&symbol).is_some() || self.reverse.contains(&symbol)
		} else {
			false
		}
	}

	fn get_bracket(&self, context: &EvalContext, node: &Node) -> Option<Self::Bracket> {
		if let Some(symbol) = node.symbol() {
			let span = node.span();
			if let Some((end, bracket_fn)) = self.pairs.get(&symbol).cloned() {
				let scope = context.scope().handle();
				Some(SymbolBracket {
					symbol,
					scope_info: Some((scope, end, bracket_fn)),
					span,
				})
			} else if self.reverse.contains(&symbol) {
				Some(SymbolBracket {
					symbol,
					scope_info: None,
					span,
				})
			} else {
				None
			}
		} else {
			None
		}
	}

	fn new_node(&self, sta: Self::Bracket, nodes: NodeList, end: Self::Bracket) -> Result<Node> {
		let span = sta.span().to(end.span());
		let (.., bracket_fn) = sta.scope_info.as_ref().unwrap();
		let node = (bracket_fn)(sta.symbol.clone(), nodes, end.symbol.clone());
		Ok(node.at(span))
	}
}

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
}

impl IsNodeOperator for BracketPairs {
	fn can_apply(&self, nodes: &NodeList) -> bool {
		nodes.has_brackets(self)
	}

	fn apply(&self, nodes: &mut NodeList, ctx: &mut EvalContext) -> Result<()> {
		nodes.parse_brackets(self, ctx)
	}
}

//====================================================================================================================//
// SymbolBracket
//====================================================================================================================//

#[derive(Clone)]
pub struct SymbolBracket {
	symbol: Symbol,
	scope_info: Option<(ScopeHandle, Symbol, BracketFn)>,
	span: Span,
}

impl IsBracket for SymbolBracket {
	fn opens(&self) -> Option<ScopeHandle> {
		self.scope_info.as_ref().map(|x| x.0.clone())
	}

	fn closes(&self, other: &Self) -> bool {
		if let Some((_, ref closing, _)) = other.scope_info {
			closing == &self.symbol
		} else {
			false
		}
	}

	fn span(&self) -> Span {
		self.span.clone()
	}
}

impl Display for SymbolBracket {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		write!(f, "`{}`", self.symbol)
	}
}

//====================================================================================================================//
// BracketPairs traits
//====================================================================================================================//

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
