use super::*;

pub struct OpIf {
	symbol_if: Symbol,
}

impl OpIf {
	pub fn new(symbol_if: Symbol) -> Self {
		Self { symbol_if }
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
}

impl IsNodeOperator for OpIf {
	fn apply(&self, ctx: &mut EvalContext, nodes: &mut NodeList) -> Result<()> {
		let _ = ctx;
		let mut new_nodes = Vec::new();
		for node in nodes.iter() {
			if let Some((head, body)) = self.get_if(&node) {
				let span = Span::merge(head.span(), body.span());
				let cond = head.slice(1..);

				ctx.del_segment(&head);
				ctx.add_segment(&cond);

				let node = Bit::If {
					condition: cond,
					when_true: body.clone(),
					when_false: None,
				};

				let node = node.at(span);
				new_nodes.push(node);
			} else {
				new_nodes.push(node);
			}
		}

		nodes.replace_all(new_nodes);
		Ok(())
	}

	fn can_apply(&self, nodes: &NodeList) -> bool {
		nodes.contains(|node| self.get_if(node).is_some())
	}
}
