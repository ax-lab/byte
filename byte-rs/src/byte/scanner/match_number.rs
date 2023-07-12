use super::*;

pub struct IntegerMatcher;

impl IsMatcher for IntegerMatcher {
	fn try_match(&self, cursor: &mut Span, errors: &mut Errors) -> Option<(Token, Span)> {
		let _ = errors;
		let start = cursor.clone();
		match cursor.read() {
			Some(next @ '0'..='9') => {
				let mut value = digit_value(next);
				let mut pos;
				let mut float = false;
				let mut dot = false;
				let mut exp = false;
				loop {
					pos = cursor.clone();
					match cursor.read() {
						Some(next @ '0'..='9') => {
							value = value * 10 + digit_value(next);
						}
						Some(c) => {
							dot = c == '.';
							exp = matches!(c, 'e' | 'E');
							break;
						}
						_ => {
							break;
						}
					}
				}

				if dot {
					float = match cursor.read() {
						Some('0'..='9') => {
							pos = cursor.clone();
							while let Some('0'..='9') = cursor.read() {
								pos = cursor.clone();
							}

							*cursor = pos.clone();
							exp = matches!(cursor.read(), Some('e' | 'E'));
							true
						}
						_ => false,
					}
				}

				if exp {
					float = true;
					let _ = cursor.read_if('+') || cursor.read_if('-');
					let mut valid = false;
					while let Some('0'..='9') = cursor.read() {
						pos = cursor.clone();
						valid = true;
					}

					if !valid {
						errors.add("invalid float literal", pos.span_from(&start));
					}
				}

				*cursor = pos;

				let span = cursor.span_from(&start);
				let token = if float {
					Token::Float(span.text().into())
				} else {
					Token::Integer(value)
				};
				Some((token, span))
			}

			_ => None,
		}
	}
}
