use std::io::Write;

use crate::core::repr::*;

use super::*;

/// Node for an expression with an indented block.
#[derive(Clone)]
pub struct BlockExpr {
	expr: Node,
	block: Node,
}

impl BlockExpr {
	pub fn new(expr: Node, block: Node) -> Node {
		let span = Span::from_range(expr.span(), block.span());
		let node = BlockExpr { expr, block };
		Node::new(node).at(span)
	}
}

has_traits!(BlockExpr: IsNode, HasRepr);

impl IsNode for BlockExpr {
	fn eval(&self, _node: Node) -> NodeEval {
		let mut done = NodeEval::Complete;
		done.check(&self.expr);
		done.check(&self.block);
		done
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
	pub fn new(nodes: Vec<Node>) -> Node {
		let sta = nodes.first();
		let end = nodes.last();
		let span = Span::from_range(sta.and_then(|x| x.span()), end.and_then(|x| x.span()));
		let node = Block { nodes };
		Node::new(node).at(span)
	}
}

has_traits!(Block: IsNode, HasRepr);

impl IsNode for Block {
	fn eval(&self, _node: Node) -> NodeEval {
		NodeEval::depends_on(&self.nodes)
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
