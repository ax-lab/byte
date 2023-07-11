use super::*;

// TODO: use a symbol as parameter and unify splits

pub struct CommaOperator;

impl IsEvaluator for CommaOperator {
	fn predicate(&self, node: &Node) -> bool {
		if let Bit::Token(Token::Symbol(symbol)) = node.bit() {
			symbol == ","
		} else {
			false
		}
	}

	fn apply(&self, nodes: &mut NodeList, context: &mut EvalContext) -> Result<()> {
		let items = nodes.split_by_items(|n| self.predicate(n));

		// FIXME: properly handle dangling commas
		for it in items.iter() {
			context.resolve_nodes(it);
		}

		let node = Bit::Sequence(items).at(nodes.span());
		nodes.replace_all(vec![node]);

		Ok(())
	}
}
