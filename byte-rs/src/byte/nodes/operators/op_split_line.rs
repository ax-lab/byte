use super::*;

pub struct OpSplitLine;

impl IsNodeOperator for OpSplitLine {
	fn applies(&self, node: &Node) -> bool {
		node.contains(|node| matches!(node.token(), Some(Token::Break(..))))
	}

	fn execute(&self, ctx: &mut OperatorContext, node: &mut Node) -> Result<()> {
		/*
			Split nodes by line while grouping by indentation.

			This will accumulate nodes in the current sequence until a line
			break is found.

			Positive indentation changes will start a new group of nodes,
			while negative indentation changes will close groups with a higher
			indentation level.

			Nested groups are merged as a single sequence.
		*/

		let mut errors = Errors::new();
		let mut lines = Vec::<Vec<Node>>::new();
		let mut empty = true;
		let mut base_level = None;
		let mut line_indent = 0;

		for child in node.iter() {
			let is_comment = matches!(child.token(), Some(Token::Comment));
			if let Some(Token::Break(indent)) = child.token() {
				// start a new line
				empty = true;
				line_indent = indent;
			} else if empty {
				// process the indentation level for a new line
				let base_level = match base_level {
					None => {
						// establish a base level for the entire block
						base_level = Some(child.indent());
						line_indent = child.indent();
						// push the first line
						lines.push(Vec::new());
						line_indent
					}
					Some(level) if line_indent < level => {
						errors.add(format!("invalid indentation"), child.span());
						level
					}
					Some(level) => level,
				};

				// indenting a line beyond base level will continue the
				// previous one, otherwise we start a new line
				if line_indent == base_level {
					lines.push(Vec::new());
				}

				if !is_comment {
					lines.last_mut().unwrap().push(child);
				}
				empty = false;
			} else if !is_comment {
				lines.last_mut().unwrap().push(child);
			}
		}

		let new_nodes = lines.into_iter().filter(|nodes| nodes.len() > 0).map(|nodes| {
			let node = Node::raw(nodes, ctx.scope_handle());
			node
		});

		node.replace_all(new_nodes.collect());
		if errors.len() > 0 {
			Err(errors)
		} else {
			Ok(())
		}
	}
}
