use super::*;

pub struct OpParseBlocks(pub Symbol);

impl OpParseBlocks {
	fn find_block(&self, node: &Node, offset: usize) -> Option<(usize, usize, usize, usize)> {
		enum State {
			Start,
			Symbol { indent: usize, pivot: usize },
			Break { indent: usize, pivot: usize },
			Body { indent: usize, pivot: usize },
		}

		let mut state = State::Start;
		let mut line_start = offset;
		let mut was_break = false;
		for (n, it) in node.iter().enumerate().skip(offset) {
			state = match state {
				State::Start => {
					if was_break {
						line_start = n;
					}
					if self.is_symbol(&it) {
						State::Symbol {
							indent: it.indent(),
							pivot: n,
						}
					} else {
						State::Start
					}
				}
				State::Symbol { indent, pivot } => {
					if matches!(it.token(), Some(Token::Break(..))) {
						State::Break { indent, pivot }
					} else if self.is_symbol(&it) {
						State::Symbol {
							indent: it.indent(),
							pivot,
						}
					} else {
						State::Start
					}
				}
				State::Break { indent, pivot } => {
					if matches!(it.token(), Some(Token::Break(..))) {
						State::Break { indent, pivot } // ignore empty lines
					} else if it.indent() > indent {
						State::Body { indent, pivot }
					} else if self.is_symbol(&it) {
						State::Symbol {
							indent: it.indent(),
							pivot,
						}
					} else {
						State::Start
					}
				}
				State::Body { indent, pivot } => {
					if !was_break || it.indent() > indent {
						State::Body { indent, pivot }
					} else {
						// don't include the line break in the block or else
						// it will get merged to the next line by the line
						// operator
						return Some((line_start, pivot, pivot + 2, n - 1));
					}
				}
			};

			was_break = matches!(it.token(), Some(Token::Break(..)));
		}

		if let State::Body { pivot, .. } = state {
			Some((line_start, pivot, pivot + 2, node.len()))
		} else {
			None
		}
	}

	fn is_symbol(&self, node: &Node) -> bool {
		node.symbol().as_ref() == Some(&self.0)
	}
}

impl IsNodeOperator for OpParseBlocks {
	fn can_apply(&self, node: &Node) -> bool {
		self.find_block(node, 0).is_some()
	}

	fn eval(&self, ctx: &mut OperatorContext, node: &mut Node) -> Result<()> {
		let mut offset = 0;
		let mut new_nodes = Vec::new();
		while let Some((start, pivot, body, end)) = self.find_block(node, offset) {
			let head = node.slice(start..pivot);
			let body = node.slice(body..end);
			assert!(head.len() > 0);
			assert!(body.len() > 0);

			new_nodes.extend(node.slice(offset..start).iter());
			offset = end;

			ctx.add_new_node(&head);
			ctx.add_new_node(&body);

			let span = Span::merge(head.span(), body.span());
			new_nodes.push(NodeValue::Block(head, body).at(ctx.scope_handle(), span));
		}
		new_nodes.extend(node.slice(offset..).iter());
		node.replace_all(new_nodes);
		Ok(())
	}
}
