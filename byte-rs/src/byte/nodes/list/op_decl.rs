use super::*;

pub struct OpDecl(pub Symbol, pub Symbol);

impl IsNodeOperator for OpDecl {
	fn can_apply(&self, nodes: &NodeList) -> bool {
		nodes.can_fold(self)
	}

	fn apply(&self, nodes: &mut NodeList, context: &mut EvalContext) -> Result<()> {
		nodes.fold(self, context)
	}
}

impl NodeFold for OpDecl {
	fn fold_at(&self, nodes: &NodeList) -> Option<usize> {
		if nodes.is_keyword(0, &self.0) && nodes.is_identifier(1) && nodes.is_symbol(2, &self.1) {
			Some(2)
		} else {
			None
		}
	}

	fn new_node(&self, context: &mut EvalContext, lhs: NodeList, rhs: NodeList, span: Span) -> Result<Node> {
		let name = lhs.get_symbol(lhs.len() - 1).unwrap();
		let offset = lhs.offset();
		let value = BindingValue::NodeList(rhs.clone());
		context.declare_at(name.clone(), offset, value);
		Ok(Bit::Let(name, offset, rhs).at(span))
	}
}
