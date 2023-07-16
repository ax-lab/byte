use super::*;

pub trait ParseTernary {
	fn delimiters(&self) -> (&Symbol, &Symbol);

	fn new_node(&self, ctx: &mut OperatorContext, a: Node, b: Node, c: Node, span: Span) -> Result<Node>;
}

impl Node {
	pub fn has_ternary<T: ParseTernary>(&self, op: &T) -> bool {
		let (sta, end) = op.delimiters();
		let nodes = self.iter();
		let mut nodes = nodes.skip_while(|x| x.symbol().as_ref() != Some(sta));
		if let Some(..) = nodes.next() {
			let mut nodes = nodes.skip_while(|x| x.symbol().as_ref() != Some(end));
			nodes.next().is_some()
		} else {
			false
		}
	}

	pub fn parse_ternary<T: ParseTernary>(&mut self, ctx: &mut OperatorContext, op: &T) -> Result<()> {
		let (sta, end) = op.delimiters();
		for i in (0..self.len()).rev() {
			if self.is_symbol_at(i, sta) {
				for j in i + 1..self.len() {
					if self.is_symbol_at(j, end) {
						let a = self.slice(0..i).to_vec();
						let b = self.slice(i + 1..j).to_vec();
						let c = self.slice(j + 1..).to_vec();
						let a = Node::raw(a, self.scope_handle());
						let b = Node::raw(b, self.scope_handle());
						let c = Node::raw(c, self.scope_handle());
						let node = op.new_node(ctx, a, b, c, self.span())?;
						self.set_value(node.val(), node.span());
						return Ok(());
					}
				}
			}
		}
		Ok(())
	}
}
