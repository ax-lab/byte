use super::*;

pub struct OpFor(pub Symbol, pub Symbol, pub Symbol);

impl OpFor {
	fn get_for(&self, node: &Node) -> Option<(NodeList, NodeList)> {
		if let NodeValue::Block(head, body) = node.val() {
			if head.is_symbol(0, &self.0) {
				Some((head, body))
			} else {
				None
			}
		} else {
			None
		}
	}
}

impl ParseReplace for OpFor {
	fn can_replace(&self, node: &Node) -> bool {
		self.get_for(node).is_some()
	}

	fn replace(&self, ctx: &mut EvalContext, node: &Node) -> Result<Option<Node>> {
		if let Some((head, body)) = self.get_for(node) {
			let span = Span::merge(head.span(), body.span());
			let mut errors = Errors::new();
			ctx.del_segment(&head);
			let node = if let Some(var) = head.get_symbol(1) {
				let var_node = head.get(1).unwrap();
				if head.is_keyword(2, &self.1) {
					let mut split = None;
					for (n, it) in head.slice(3..).iter().enumerate() {
						if it.is_symbol(&self.2) {
							split = Some(n);
							break;
						}
					}
					if let Some(split) = split {
						let from = head.slice(3..3 + split);
						let to = head.slice(3 + split + 1..);
						if from.len() == 0 {
							errors.add("missing lower bound in `for`", head.span());
						}
						if to.len() == 0 {
							errors.add("missing upper bound in `for`", head.span());
						}

						// TODO: this for binding is completely bogus, figure out a better way
						let offset = var_node.offset();
						ctx.declare_at(var.clone(), offset, BindingValue::NodeList(from.clone()));

						ctx.add_segment(&from);
						ctx.add_segment(&to);
						let body = body.clone();
						let node = NodeValue::For {
							var,
							offset,
							from,
							to,
							body,
						}
						.at(ctx.scope_handle(), span);
						Some(node)
					} else {
						None
					}
				} else {
					None
				}
			} else {
				None
			};
			if node.is_none() {
				errors.add("invalid `for` syntax", head.span());
			}
			if errors.len() > 0 {
				Err(errors)
			} else {
				Ok(node)
			}
		} else {
			Ok(None)
		}
	}
}

impl IsNodeOperator for OpFor {
	fn apply(&self, ctx: &mut EvalContext, nodes: &mut NodeList) -> Result<()> {
		nodes.replace(ctx, self)
	}

	fn can_apply(&self, nodes: &NodeList) -> bool {
		nodes.can_replace(self)
	}
}
