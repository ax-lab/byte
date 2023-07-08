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
				loop {
					pos = cursor.clone();
					match cursor.read() {
						Some(next @ '0'..='9') => {
							value = value * 10 + digit_value(next);
						}
						_ => {
							break;
						}
					}
				}
				*cursor = pos;
				Some((Token::Integer(value), cursor.span_from(&start)))
			}

			_ => None,
		}
	}
}
