use crate::core::error::*;
use crate::core::input::*;
use crate::lexer::*;

pub struct Integer;

impl IsToken for Integer {
	type Value = u64;

	fn name() -> &'static str {
		"Number"
	}
}

impl Matcher for Integer {
	fn try_match(&self, next: char, input: &mut Cursor, _errors: &mut ErrorList) -> Option<Token> {
		match next {
			'0'..='9' => {
				let mut value = decimal_value(next);
				let mut pos;
				loop {
					pos = input.clone();
					match input.read() {
						Some(next @ '0'..='9') => {
							value = value * 10 + decimal_value(next);
						}
						_ => {
							break;
						}
					}
				}
				*input = pos;
				Some(Integer::token(value))
			}

			_ => None,
		}
	}
}

fn decimal_value(n: char) -> u64 {
	(n as u64) - ('0' as u64)
}
