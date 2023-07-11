use super::*;

pub struct OpPrint(pub Symbol);

impl IsNodeOperator for OpPrint {
	fn can_apply(&self, nodes: &NodeList) -> bool {
		nodes.has_keyword(self)
	}

	fn apply(&self, nodes: &mut NodeList, ctx: &mut EvalContext) -> Result<()> {
		nodes.parse_keyword(self, ctx)
	}
}

impl NodeKeyword for OpPrint {
	fn symbol(&self) -> &Symbol {
		&self.0
	}

	fn new_node(&self, args: NodeList, span: Span) -> Result<Node> {
		Ok(Bit::Print(args, "\n").at(span))
	}
}
