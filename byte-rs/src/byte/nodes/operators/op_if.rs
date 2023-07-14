use super::*;

pub struct OpIf {
	symbol_if: Symbol,
	symbol_else: Symbol,
}

impl OpIf {
	pub fn new(symbol_if: Symbol, symbol_else: Symbol) -> Self {
		Self { symbol_if, symbol_else }
	}

	fn get_if(&self, node: &Node) -> Option<(Node, Node)> {
		if let NodeValue::Block(head, body) = node.val() {
			if head.is_symbol_at(0, &self.symbol_if) {
				Some((head, body))
			} else {
				None
			}
		} else {
			None
		}
	}

	fn get_else(&self, node: &Node) -> Option<(Node, Node, bool)> {
		match node.val() {
			NodeValue::Block(head, body) => {
				if head.is_symbol_at(0, &self.symbol_else) {
					let is_if = head.is_symbol_at(1, &self.symbol_if);
					Some((head.clone(), body.clone(), is_if))
				} else {
					None
				}
			}
			_ => None,
		}
	}
}

impl IsNodeOperator for OpIf {
	fn can_apply(&self, node: &Node) -> bool {
		node.contains(|node| self.get_if(node).is_some())
	}

	fn eval(&self, ctx: &mut EvalContext, node: &mut Node) -> Result<()> {
		let mut new_nodes = Vec::new();

		let mut errors = Errors::new();

		let mut n = 0;
		while n < node.len() {
			let node = node.get(n).unwrap();
			if let Some((head, body)) = self.get_if(&node) {
				let span = Span::merge(head.span(), body.span());
				let cond = head.slice(1..);

				ctx.forget_node(&head);
				ctx.add_new_node(&cond);

				let mut else_ifs = VecDeque::new();
				let mut if_false = None;

				while if_false.is_none() {
					// skip line breaks because the line operator hasn't run yet
					let mut m = n + 1;
					while let Some(Token::Break(..)) = node.get(m).and_then(|x| x.token()) {
						m += 1;
					}

					if let Some(node) = node.get(m) {
						if let Some((head, body, is_if)) = self.get_else(&node) {
							n = m;
							ctx.forget_node(&head);
							if is_if {
								let head = head.slice(2..);
								if head.len() == 0 {
									errors.add("if condition missing", head.span());
								}
								ctx.add_new_node(&head);
								else_ifs.push_back((head, body));
							} else if head.len() == 1 {
								if_false = Some(body);
							} else {
								let head = head.slice(1..);
								ctx.add_new_node(&head);
								let span = Span::merge(head.span(), body.span());
								let node = NodeValue::Block(head, body.clone()).at(ctx.scope_handle(), span);
								let node = node.to_raw();
								ctx.add_new_node(&node);
								if_false = Some(node);
							}
						} else {
							break;
						}
					} else {
						break;
					};
				}

				while let Some((if_cond, if_body)) = else_ifs.pop_back() {
					let span = Span::merge(if_cond.span(), if_body.span());
					let node = NodeValue::If {
						expr: if_cond,
						if_true: if_body,
						if_false,
					};
					let node = node.at(ctx.scope_handle(), span);
					let nodes = Node::raw(vec![node], ctx.scope_handle());
					ctx.add_new_node(&nodes);
					if_false = Some(nodes);
				}

				let node = NodeValue::If {
					expr: cond,
					if_true: body.clone(),
					if_false,
				};

				let node = node.at(ctx.scope_handle(), span);
				new_nodes.push(node);
			} else {
				new_nodes.push(node);
			}

			n += 1;
		}

		node.replace_all(new_nodes);
		if errors.len() > 0 {
			Err(errors)
		} else {
			Ok(())
		}
	}
}
