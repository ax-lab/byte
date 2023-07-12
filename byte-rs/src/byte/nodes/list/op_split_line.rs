use super::*;

pub struct OpSplitLine;

impl IsNodeOperator for OpSplitLine {
	fn can_apply(&self, nodes: &NodeList) -> bool {
		nodes.contains(|node| matches!(node.token(), Some(Token::Break)))
	}

	fn apply(&self, ctx: &mut EvalContext, nodes: &mut NodeList) -> Result<()> {
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
		for node in nodes.iter() {
			if let Some(Token::Break) = node.token() {
				// start a new line, skipping blank lines
				if !empty {
					empty = true;
				}
			} else if empty {
				// process the indentation level for a new line
				let new_level = node.indent();
				let base_level = match base_level {
					None => {
						// establish a base level for the entire block
						base_level = Some(new_level);
						// push the first line
						lines.push(Vec::new());
						new_level
					}
					Some(level) if new_level < level => {
						errors.add(format!("invalid indentation"), node.span());
						level
					}
					Some(level) => level,
				};

				// indenting a line beyond base level will continue the
				// previous one, otherwise we start a new line
				if new_level == base_level {
					lines.push(Vec::new());
				}

				lines.last_mut().unwrap().push(node);
				empty = false;
			} else {
				lines.last_mut().unwrap().push(node);
			}
		}

		let new_nodes = lines.into_iter().filter(|nodes| nodes.len() > 0).map(|nodes| {
			let nodes = NodeList::new(ctx.scope_handle(), nodes);
			let span = nodes.span();
			ctx.resolve_nodes(&nodes);
			Bit::Line(nodes).at(span)
		});

		nodes.replace_all(new_nodes.collect());
		if errors.len() > 0 {
			Err(errors)
		} else {
			Ok(())
		}
	}
}
