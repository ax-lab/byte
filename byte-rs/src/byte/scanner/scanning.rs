use super::*;

pub fn scan(scope: &mut ScopeWriter, input: &Span) -> Result<NodeList> {
	let mut matcher = scope.matcher();
	let mut errors = Errors::new();
	let mut nodes = Vec::new();
	let mut cursor = input.clone();
	while let Some((token, span)) = matcher.scan(&mut cursor, &mut errors) {
		nodes.push(NodeValue::Token(token).at(scope.handle(), span));
		if !errors.empty() {
			break;
		}
	}

	scope.set_matcher(matcher);

	if !cursor.at_end() && errors.empty() {
		errors.add("failed to parse the entire input", cursor.pos());
	}

	if errors.len() > 0 {
		Err(errors)
	} else {
		let nodes = NodeList::new(scope.handle(), nodes);
		Ok(nodes)
	}
}
