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
	fn applies(&self, node: &Node) -> bool {
		node.can_fold(self)
	}

	fn execute(&self, ctx: &mut OperatorContext, node: &mut Node) -> Result<()> {
		node.fold(ctx, self)
	}
}

impl ParseFold for OpDecl {
	fn fold_at(&self, node: &Node) -> Option<usize> {
		if node.is_keyword_at(0, &self.0) && node.is_identifier(1) && node.is_symbol_at(2, &self.1) {
			Some(2)
		} else {
			None
		}
	}

	fn new_node(&self, ctx: &mut OperatorContext, lhs: Node, rhs: Node, span: Span) -> Result<Node> {
		let name = lhs.get_symbol_at(lhs.len() - 1).unwrap();
		let value = BindingValue::Node(rhs.clone());
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
