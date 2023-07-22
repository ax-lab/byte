use super::*;

// TODO: this should be a "macro" style binding that evaluates to a `Let` expression

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Decl {
	Let,
	Const,
}

pub struct EvalDecl(pub Symbol, pub Symbol, pub Decl);

impl EvalDecl {
	pub fn mode(&self) -> Decl {
		self.2
	}
}

impl IsNodeEval for EvalDecl {
	fn applies(&self, node: &Node) -> bool {
		node.can_fold(self)
	}

	fn execute(&self, ctx: &mut EvalContext, node: &mut Node) -> Result<()> {
		node.fold(ctx, self)
	}
}

impl ParseFold for EvalDecl {
	fn fold_at(&self, node: &Node) -> Option<usize> {
		if node.is_keyword_at(0, &self.0) && node.is_identifier(1) && node.is_symbol_at(2, &self.1) {
			Some(2)
		} else {
			None
		}
	}

	fn new_node(&self, ctx: &mut EvalContext, lhs: Node, rhs: Node, span: Span) -> Result<Node> {
		let name = lhs.get_symbol_at(lhs.len() - 1).unwrap();
		let value = rhs.clone();
		let offset = if self.mode() == Decl::Const {
			CodeOffset::Static
		} else {
			let offset = lhs.offset();
			CodeOffset::At(offset)
		};

		ctx.declare(name.clone(), offset, value);
		Ok(Expr::Let(name, offset, rhs).at(ctx.scope_handle(), span))
	}
}
