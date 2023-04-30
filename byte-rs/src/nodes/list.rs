use std::io::Write;

use crate::core::*;

use super::*;

#[derive(Clone)]
pub struct List {
	items: Vec<Node>,
}

impl List {
	pub fn from(mut head: Node, separator: &'static str) -> Node {
		let mut list = Vec::new();
		let mut curr = head.clone();
		loop {
			let sep = curr
				.get::<Atom>()
				.map(|x| x.symbol() == Some(separator))
				.unwrap_or(false);
			if sep {
				let next = curr.split_next();
				if curr == head {
					// TODO: empty item? check hanging separator
				} else {
					curr.extract();
					list.push(head);
				}
				if let Some(next) = next {
					head = next;
					curr = head.clone();
				} else {
					break;
				}
			} else {
				if let Some(next) = curr.next() {
					curr = next;
				} else {
					break;
				}
			}
		}

		let span = Span::from_range(
			list.first().and_then(|x| x.span()),
			list.last().and_then(|x| x.span()),
		);
		let node = List { items: list };
		Node::new(node).at(span)
	}

	pub fn empty(at: Option<Span>) -> Node {
		let node = List { items: Vec::new() };
		Node::new(node).at(at)
	}
}

has_traits!(List: IsNode, HasRepr);

impl IsNode for List {
	fn eval(&self, _node: Node) -> NodeEval {
		NodeEval::depends_on(&self.items)
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
