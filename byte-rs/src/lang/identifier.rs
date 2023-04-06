use crate::core::error::*;
use crate::core::input::*;
use crate::lexer::*;

pub struct Identifier;

impl Matcher for Identifier {
	fn try_match(&self, next: char, input: &mut Cursor, _errors: &mut ErrorList) -> Option<Token> {
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
