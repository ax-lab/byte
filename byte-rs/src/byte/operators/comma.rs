use super::*;

pub struct CommaOperator;

impl Evaluator for CommaOperator {
	fn predicate(&self, node: &Node) -> bool {
		if let Bit::Token(Token::Symbol(symbol)) = node.bit() {
			symbol == ","
		} else {
			false
		}
	}

	fn apply(&self, scope: &Scope, nodes: &mut Vec<Node>, context: &mut EvalContext) -> Result<bool> {
		let items = Nodes::split_by_items(scope, nodes, |n| self.predicate(n));
		if items.len() == 1 {
			return Ok(false);
		} else {
			for it in items.iter() {
				context.resolve_nodes(it);
			}

			let node = Bit::Sequence(items).at(context.span());
			*nodes = vec![node];
			Ok(true)
		}
	}
}
