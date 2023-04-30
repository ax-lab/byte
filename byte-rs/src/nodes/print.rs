use std::io::Write;

use crate::core::*;

use super::*;

#[derive(Clone)]
pub struct Print {
	args: Node,
}

impl Print {
	pub fn new(head: Node, args: Node) -> Node {
		let span = Span::from_range(head.span(), args.span());
		let node = Print { args };
		Node::new(node).at(span)
	}
}

has_traits!(Print: IsNode, IsExprValueNode, HasRepr);

impl IsNode for Print {
	fn eval(&self, _node: Node) -> NodeEval {
		let mut done = NodeEval::Complete;
		done.check(&self.args);
		done
	}
}

impl IsExprValueNode for Print {
	fn is_value(&self) -> bool {
		true
	}
}

impl HasRepr for Print {
	fn output_repr(&self, output: &mut Repr) -> std::io::Result<()> {
		let full = output.is_full();
		let debug = output.is_debug();
		if debug {
			write!(output, "Print(")?;
		} else {
			write!(output, "print(")?;
		}
		if full {
			write!(output, "\n")?;
		}

		{
			let mut output = output.indented();
			self.args.output_repr(&mut output)?;
		}

		if debug {
			if full {
				write!(output, "\n")?;
			}
			write!(output, ")")?;
		}

		Ok(())
	}
}

//====================================================================================================================//
// Print macro
//====================================================================================================================//

#[derive(Clone)]
pub struct PrintMacro;

has_traits!(PrintMacro: IsNode, HasRepr);

impl IsNode for PrintMacro {
	fn eval(&self, mut node: Node) -> NodeEval {
		let (list, span) = if let Some(next) = node.split_next() {
			(List::from(next, ","), None)
		} else {
			(List::empty(None), node.span())
		};
		let print = Print::new(node.clone(), list);
		node.set_value_from_node(&print);
		node.set_span(span);
		NodeEval::Changed
	}
}

impl HasRepr for PrintMacro {
	fn output_repr(&self, output: &mut Repr) -> std::io::Result<()> {
		write!(output, "<PrintMacro>")
	}
}
