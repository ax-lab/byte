use super::*;

pub struct OpIf {
	symbol_if: Symbol,
	symbol_else: Symbol,
}

impl OpIf {
	pub fn new(symbol_if: Symbol, symbol_else: Symbol) -> Self {
		Self { symbol_if, symbol_else }
	}

	fn get_if<'a>(&self, node: &'a Node) -> Option<(&'a NodeList, &'a NodeList)> {
		if let Bit::Block(head, body) = node.bit() {
			if head.is_symbol(0, &self.symbol_if) {
				Some((head, body))
			} else {
				None
			}
		} else {
			None
		}
	}

	fn get_else<'a>(&self, node: &Node) -> Option<(NodeList, NodeList, bool)> {
		match node.bit() {
			Bit::Block(head, body) => {
				if head.is_symbol(0, &self.symbol_else) {
					let is_if = head.is_symbol(1, &self.symbol_if);
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
	fn apply(&self, ctx: &mut EvalContext, nodes: &mut NodeList) -> Result<()> {
		let _ = ctx;
		let mut new_nodes = Vec::new();

		let mut errors = Errors::new();

		let mut n = 0;
		while n < nodes.len() {
			let node = nodes.get(n).unwrap();
			if let Some((head, body)) = self.get_if(&node) {
				let span = Span::merge(head.span(), body.span());
				let cond = head.slice(1..);

				ctx.del_segment(&head);
				ctx.add_segment(&cond);

				let mut else_ifs = VecDeque::new();
				let mut when_false = None;

				while when_false.is_none() {
					// skip line breaks because the line operator hasn't run yet
					let mut m = n + 1;
					while let Some(Token::Break(..)) = nodes.get(m).and_then(|x| x.token().cloned()) {
						m += 1;
					}

					if let Some(node) = nodes.get(m) {
						if let Some((head, body, is_if)) = self.get_else(&node) {
							n = m;
							ctx.del_segment(&head);
							if is_if {
								let head = head.slice(2..);
								if head.len() == 0 {
									errors.add("if condition missing", head.span());
								}
								ctx.add_segment(&head);
								else_ifs.push_back((head, body));
							} else if head.len() == 1 {
								when_false = Some(body);
							} else {
								let head = head.slice(1..);
								ctx.add_segment(&head);
								let span = Span::merge(head.span(), body.span());
								let node = Bit::Block(head, body.clone()).at(span);
								let nodes = NodeList::new(ctx.scope_handle(), vec![node]);
								ctx.add_segment(&nodes);
								when_false = Some(nodes);
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
					let node = Bit::If {
						condition: if_cond,
						when_true: if_body,
						when_false,
					};
					let node = node.at(span);
					let nodes = NodeList::new(ctx.scope_handle(), vec![node]);
					ctx.add_segment(&nodes);
					when_false = Some(nodes);
				}

				let node = Bit::If {
					condition: cond,
					when_true: body.clone(),
					when_false,
				};

				let node = node.at(span);
				new_nodes.push(node);
			} else {
				new_nodes.push(node);
			}

			n += 1;
		}

		nodes.replace_all(new_nodes);
		if errors.len() > 0 {
			Err(errors)
		} else {
			Ok(())
		}
	}

	fn can_apply(&self, nodes: &NodeList) -> bool {
		nodes.contains(|node| self.get_if(node).is_some())
	}
}
