use super::*;

pub trait ParseBrackets {
	type Bracket: IsBracket;

	fn is_bracket(&self, node: &Node) -> bool;

	fn get_bracket(&self, ctx: &EvalContext, node: &Node) -> Option<Self::Bracket>;

	fn new_node(&self, ctx: &mut EvalContext, sta: Self::Bracket, nodes: Node, end: Self::Bracket) -> Result<Node>;
}

pub trait IsBracket: Clone + Display {
	fn opens(&self) -> Option<ScopeHandle>;

	fn closes(&self, other: &Self) -> bool;

	fn span(&self) -> Span;
}

impl Node {
	pub fn has_brackets<T: ParseBrackets>(&self, op: &T) -> bool {
		self.contains(|x| op.is_bracket(x))
	}

	pub fn parse_brackets<T: ParseBrackets>(&mut self, ctx: &mut EvalContext, op: &T) -> Result<()> {
		self.write_res(|nodes| {
			let mut has_brackets = false;
			let mut segments = VecDeque::new();
			segments.push_back(Vec::new());

			let mut scopes = VecDeque::new();
			let mut stack = VecDeque::<T::Bracket>::new();
			for node in nodes.iter() {
				if let Some(cur) = op.get_bracket(ctx, node) {
					if let Some(start) = stack.back() {
						if cur.closes(start) {
							let start = stack.pop_back().unwrap();
							let scope = scopes.pop_back().unwrap();
							let nodes = segments.pop_back().unwrap();
							let nodes = Node::raw(nodes, scope);
							let node = op.new_node(ctx, start, nodes, cur)?;
							node.get_dependencies(|list| ctx.add_new_node(list));
							segments.back_mut().unwrap().push(node);
							continue;
						}
					}

					if let Some(scope) = cur.opens() {
						has_brackets = true;
						stack.push_back(cur);
						scopes.push_back(scope);
						segments.push_back(Default::default());
					} else {
						let error = format!("unpaired closing bracket {cur}");
						let error = Errors::from(error, cur.span());
						return Err(error);
					}
				} else {
					segments.back_mut().unwrap().push(node.clone());
				}
			}

			if let Some(open) = stack.pop_back() {
				let error = format!("expected a closing bracket for open bracket {open}");
				let error = Errors::from(error, open.span());
				return Err(error);
			}

			let changed = if has_brackets {
				*nodes = segments.pop_back().unwrap();
				true
			} else {
				false
			};
			Ok(changed)
		})
	}
}
