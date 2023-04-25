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
	pub fn new(sta: TokenAt, end: TokenAt, node: Node) -> Self {
		Group { sta, end, node }
	}
}

has_traits!(Group: IsNode, HasRepr, IsExprValueNode);

impl IsNode for Group {
	fn eval(&mut self, _scope: &mut Scope) -> NodeEval {
		if self.node.is_done() {
			NodeEval::Complete
		} else {
			NodeEval::DependsOn(vec![self.node.clone()])
		}
	}

	fn span(&self) -> Option<Span> {
		Span::from_range(Some(self.sta.span()), Some(self.end.span()))
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
