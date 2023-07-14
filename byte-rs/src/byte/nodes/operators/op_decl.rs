use super::*;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Decl {
	Let,
	Const,
}

pub struct OpDecl(pub Symbol, pub Symbol, pub Decl);

impl OpDecl {
	pub fn mode(&self) -> Decl {
		self.2
	}
}

impl IsNodeOperator for OpDecl {
	fn can_apply(&self, nodes: &NodeList) -> bool {
		nodes.can_fold(self)
	}

	fn apply(&self, ctx: &mut EvalContext, nodes: &mut NodeList) -> Result<()> {
		nodes.fold(ctx, self)
	}
}

impl ParseFold for OpDecl {
	fn fold_at(&self, nodes: &NodeList) -> Option<usize> {
		if nodes.is_keyword(0, &self.0) && nodes.is_identifier(1) && nodes.is_symbol(2, &self.1) {
			Some(2)
		} else {
			None
		}
	}

	fn new_node(&self, ctx: &mut EvalContext, lhs: NodeList, rhs: NodeList, span: Span) -> Result<Node> {
		let name = lhs.get_symbol(lhs.len() - 1).unwrap();
		let value = BindingValue::NodeList(rhs.clone());
		let offset = if self.mode() == Decl::Const {
			ctx.declare_static(name.clone(), value);
			None
		} else {
			let offset = lhs.offset();
			ctx.declare_at(name.clone(), offset, value);
			Some(offset)
		};
		Ok(NodeValue::Let(name, offset, rhs).at(ctx.scope_handle(), span))
	}
}
