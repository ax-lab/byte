use std::io::Write;

use crate::core::*;

use super::*;

#[derive(Clone)]
pub struct Print {
	args: Node,
}

impl Print {
	pub fn new(args: Node) -> Self {
		Print { args }
	}
}

has_traits!(Print: IsNode, IsExprValueNode, HasRepr);

impl IsNode for Print {
	fn eval(&mut self, _scope: &mut Scope) -> NodeEval {
		let mut done = NodeEval::Complete;
		done.check(&self.args);
		done
	}

	fn span(&self) -> Option<Span> {
		self.args.span()
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

has_traits!(PrintMacro: IsNode, HasRepr, IsMacroNode);

impl IsNode for PrintMacro {
	fn eval(&mut self, _scope: &mut Scope) -> NodeEval {
		NodeEval::Complete
	}

	fn span(&self) -> Option<Span> {
		None
	}
}

impl IsMacroNode for PrintMacro {
	fn try_parse(&self, nodes: &[Node], scope: Scope) -> Option<(Vec<Node>, usize)> {
		// the first node is ourselves
		let args = List::from(&nodes[1..], ",", scope.clone());
		let node = Node::new(Print::new(args), scope);
		Some((vec![node], nodes.len()))
	}
}

impl HasRepr for PrintMacro {
	fn output_repr(&self, output: &mut Repr) -> std::io::Result<()> {
		write!(output, "<PrintMacro>")
	}
}
