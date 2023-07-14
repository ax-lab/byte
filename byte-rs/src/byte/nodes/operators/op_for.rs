use super::*;

pub struct OpFor(pub Symbol, pub Symbol, pub Symbol);

impl OpFor {
	fn get_for(&self, node: &Node) -> Option<(Node, Node)> {
		if let NodeValue::Block(head, body) = node.val() {
			if head.is_symbol_at(0, &self.0) {
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
			ctx.forget_node(&head);
			let node = if let Some(var) = head.get_symbol_at(1) {
				let var_node = head.get(1).unwrap();
				if head.is_keyword_at(2, &self.1) {
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
						ctx.declare_at(var.clone(), offset, BindingValue::Node(from.clone()));

						ctx.add_new_node(&from);
						ctx.add_new_node(&to);
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
	fn can_apply(&self, node: &Node) -> bool {
		node.can_replace(self)
	}

	fn eval(&self, ctx: &mut EvalContext, node: &mut Node) -> Result<()> {
		node.replace(ctx, self)
	}
}
