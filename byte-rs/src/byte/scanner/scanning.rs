use super::*;

pub fn scan(matcher: &mut Matcher, input: &Span) -> Result<Vec<Node>> {
	let mut errors = Errors::new();
	let mut output = Vec::new();
	let mut cursor = input.clone();
	while let Some((token, span)) = matcher.scan(&mut cursor, &mut errors) {
		output.push(Bit::Token(token).at(span));
		if !errors.empty() {
			break;
		}
	}

	if !cursor.at_end() && errors.empty() {
		errors.add("failed to parse the entire input", cursor.pos());
	}

	if errors.len() > 0 {
		Err(errors)
	} else {
		Ok(output)
	}
}
