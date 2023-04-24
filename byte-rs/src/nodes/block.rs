use std::io::Write;

use crate::core::repr::*;
use crate::lexer::*;

use super::*;

/// Node for an expression with an indented block.
#[derive(Clone)]
pub struct BlockExpr {
	expr: Node,
	block: Node,
}

impl BlockExpr {
	pub fn new(expr: Node, block: Node) -> Self {
		BlockExpr { expr, block }
	}
}

has_traits!(BlockExpr: IsNode, HasRepr);

impl IsNode for BlockExpr {
	fn eval(&mut self, scope: &mut Scope) -> NodeEval {
		let mut pending = Vec::new();
		if !self.expr.is_done() {
			pending.push(self.expr.clone());
		}
		if !self.block.is_done() {
			pending.push(self.block.clone());
		}
		if pending.len() > 0 {
			NodeEval::DependsOn(pending)
		} else {
			NodeEval::Complete
		}
	}

	fn span(&self) -> Option<Span> {
		Span::from_range(self.expr.span(), self.block.span())
	}
}

impl HasRepr for BlockExpr {
	fn output_repr(&self, output: &mut Repr) -> std::io::Result<()> {
		if output.is_debug() {
			write!(output, "BlockExpr(")?;
			{
				let mut output = output.indented();
				write!(output, "\n")?;
				self.expr.output_repr(&mut output)?;
				write!(output, "\n")?;
				self.block.output_repr(&mut output)?;
			}
			write!(output, "\n)")?;
		} else {
			self.expr.output_repr(output)?;
			write!(output, " {{\n")?;
			{
				let mut output = output.indented();
				self.block.output_repr(&mut output)?;
			}
			write!(output, "\n}}")?;
		}
		Ok(())
	}
}

/// Node for a block of statements.
#[derive(Clone)]
pub struct Block {
	nodes: Vec<Node>,
}

impl Block {
	pub fn new(nodes: Vec<Node>) -> Self {
		Block { nodes }
	}
}

has_traits!(Block: IsNode, HasRepr);

impl IsNode for Block {
	fn eval(&mut self, scope: &mut Scope) -> NodeEval {
		let pending: Vec<Node> = self
			.nodes
			.iter()
			.filter(|x| !x.is_done())
			.cloned()
			.collect();
		if pending.len() == 0 {
			NodeEval::Complete
		} else {
			NodeEval::DependsOn(pending)
		}
	}

	fn span(&self) -> Option<Span> {
		let first = self.nodes.first();
		let last = self.nodes.last();
		Span::from_range(first.and_then(|x| x.span()), last.and_then(|x| x.span()))
	}
}

impl HasRepr for Block {
	fn output_repr(&self, output: &mut Repr) -> std::io::Result<()> {
		{
			let mut output = if output.is_debug() {
				write!(output, "Block(\n")?;
				output.indented()
			} else {
				output.clone()
			};
			for (i, it) in self.nodes.iter().enumerate() {
				if i > 0 {
					write!(output, "\n",)?;
				}
				it.output_repr(&mut output)?;
			}
		}
		if output.is_debug() {
			write!(output, "\n)")?;
		}

		Ok(())
	}
}
