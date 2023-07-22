use super::*;

pub fn scan(scope: &mut ScopeWriter, input: &Span) -> Result<Node> {
	let mut matcher = scope.matcher();
	let mut errors = Errors::new();
	let mut nodes = Vec::new();
	let mut cursor = input.clone();
	while let Some((token, span)) = matcher.scan(&mut cursor, &mut errors) {
		nodes.push(Expr::Token(token).at(scope.handle(), span));
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
		let nodes = Expr::Raw(nodes.into()).at(scope.handle(), input.clone());
		Ok(nodes)
	}
}
