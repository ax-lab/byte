use super::*;

// TODO: use a symbol as parameter and unify splits

pub struct CommaOperator;

impl CommaOperator {
	pub fn is_comma(&self, node: &Node) -> bool {
		if let Bit::Token(Token::Symbol(symbol)) = node.bit() {
			symbol == ","
		} else {
			false
		}
	}
}

impl IsNodeOperator for CommaOperator {
	fn can_apply(&self, nodes: &NodeList) -> bool {
		nodes.contains(|x| self.is_comma(x))
	}

	fn apply(&self, nodes: &mut NodeList, context: &mut EvalContext) -> Result<()> {
		let items = nodes.split_by_items(|n| self.is_comma(n));

		// FIXME: properly handle dangling commas
		for it in items.iter() {
			context.resolve_nodes(it);
		}

		let node = Bit::Sequence(items).at(nodes.span());
		nodes.replace_all(vec![node]);

		Ok(())
	}
}
