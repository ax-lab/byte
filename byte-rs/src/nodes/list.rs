use std::io::Write;

use crate::core::*;

use super::*;

#[derive(Clone)]
pub struct List {
	items: Vec<Node>,
}

impl List {
	pub fn from(input: &[Node], separator: &'static str, scope: Scope) -> Node {
		let items = input.split(|x| {
			x.get::<Atom>()
				.map(|x| x.symbol() == Some(separator))
				.unwrap_or(false)
		});

		let mut last_empty = false;
		let mut items: Vec<Node> = items
			.map(|it| {
				last_empty = it.len() == 0;
				let node = Raw::new(it.to_vec());
				let node = Node::new(node, scope.clone());
				node
			})
			.collect();
		if last_empty {
			items.pop();
		}

		Node::new(List { items }, scope)
	}
}

has_traits!(List: IsNode, HasRepr);

impl IsNode for List {
	fn eval(&mut self, _scope: &mut Scope) -> NodeEval {
		NodeEval::depends_on(&self.items)
	}

	fn span(&self) -> Option<Span> {
		Node::span_from_list(&self.items)
	}
}

impl HasRepr for List {
	fn output_repr(&self, output: &mut Repr) -> std::io::Result<()> {
		let full = output.is_full();
		let debug = output.is_debug();
		if debug {
			write!(output, "List(")?;
		} else {
			write!(output, "[")?;
		};

		{
			let mut output = output.indented();
			for (i, it) in self.items.iter().enumerate() {
				if i == 0 {
					if full {
						write!(output, "\n")?;
					} else if !debug {
						write!(output, " ")?;
					}
				} else if !full {
					write!(output, ", ")?;
				}
				it.output_repr(&mut output)?;
				if full {
					write!(output, "\n")?;
				}
			}
		}

		if debug {
			write!(output, ")")?;
		} else {
			write!(output, "]")?;
		};
		Ok(())
	}
}
