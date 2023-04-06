use crate::core::input::*;

use super::*;

pub struct Identifier;

impl Matcher for Identifier {
	fn try_match(&self, next: char, input: &mut Cursor, errors: &mut ErrorList) -> Option<Token> {
		match next {
			'a'..='z' | 'A'..='Z' | '_' => {
				let mut pos;
				loop {
					pos = input.clone();
					match input.read() {
						Some('a'..='z' | 'A'..='Z' | '_' | '0'..='9') => {}
						_ => {
							*input = pos;
							break;
						}
					}
				}

				Some(Token::Identifier)
			}

			_ => None,
		}
	}
}
