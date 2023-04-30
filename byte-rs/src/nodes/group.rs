use std::io::Write;

use crate::core::repr::*;
use crate::lexer::*;

use super::*;

/// Node for parenthesized blocks and expressions.
#[derive(Clone)]
pub struct Group {
	sta: TokenAt,
	end: TokenAt,
	node: Node,
}

impl Group {
	pub fn new(sta: TokenAt, end: TokenAt, node: Node) -> Node {
		let span = Span::from_range(Some(sta.span()), Some(end.span()));
		let node = Group { sta, end, node };
		Node::new(node).at(span)
	}
}

has_traits!(Group: IsNode, HasRepr, IsExprValueNode);

impl IsNode for Group {
	fn eval(&self, _node: Node) -> NodeEval {
		if self.node.is_done() {
			NodeEval::Complete
		} else {
			NodeEval::DependsOn(vec![self.node.clone()])
		}
	}
}

impl IsExprValueNode for Group {
	fn is_value(&self) -> bool {
		true
	}
}

impl HasRepr for Group {
	fn output_repr(&self, output: &mut Repr) -> std::io::Result<()> {
		if output.is_debug() {
			write!(output, "Group")?;
		}
		write!(output, "{}\n", self.sta.symbol().unwrap_or("!?"))?;
		{
			let mut output = output.indented();
			self.node.output_repr(&mut output)?;
			write!(output, "\n")?;
		}
		write!(output, "{}", self.end.symbol().unwrap_or("?!"))?;
		Ok(())
	}
}
