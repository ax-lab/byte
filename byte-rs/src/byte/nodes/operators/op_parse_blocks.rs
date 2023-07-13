use super::*;

pub struct OpParseBlocks(pub Symbol);

impl OpParseBlocks {
	fn find_block(&self, nodes: &NodeList) -> Option<(usize, usize, usize, usize)> {
		enum State {
			Start,
			Symbol { indent: usize, pivot: usize },
			Break { indent: usize, pivot: usize },
			Body { indent: usize, pivot: usize },
		}

		let mut state = State::Start;
		let mut line_start = 0;
		let mut was_break = false;
		for (n, it) in nodes.iter().enumerate() {
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
						// don't include the line break in the block, this is
						// important to split consecutive blocks
						return Some((line_start, pivot, pivot + 2, n - 1));
					}
				}
			};

			was_break = matches!(it.token(), Some(Token::Break(..)));
		}

		if let State::Body { pivot, .. } = state {
			Some((line_start, pivot, pivot + 2, nodes.len()))
		} else {
			None
		}
	}

	fn is_symbol(&self, node: &Node) -> bool {
		node.symbol().as_ref() == Some(&self.0)
	}
}

impl IsNodeOperator for OpParseBlocks {
	fn can_apply(&self, nodes: &NodeList) -> bool {
		self.find_block(nodes).is_some()
	}

	fn apply(&self, ctx: &mut EvalContext, nodes: &mut NodeList) -> Result<()> {
		let (start, pivot, body, end) = self.find_block(nodes).unwrap();
		let head = nodes.slice(start..pivot);
		let body = nodes.slice(body..end);
		assert!(head.len() > 0);
		assert!(body.len() > 0);

		ctx.add_segment(&head);
		ctx.add_segment(&body);

		let mut new_nodes = Vec::new();
		let span = Span::merge(head.span(), body.span());
		new_nodes.extend(nodes.slice(0..start).iter());
		new_nodes.push(Bit::Block(head, body).at(span));
		new_nodes.extend(nodes.slice(end..).iter());
		nodes.replace_all(new_nodes);
		Ok(())
	}
}
