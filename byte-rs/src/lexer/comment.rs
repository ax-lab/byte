use crate::core::error::*;
use crate::core::input::*;

use super::*;

pub struct Comment;

impl IsToken for Comment {
	type Value = ();

	fn name() -> &'static str {
		"comment"
	}
}

impl Matcher for Comment {
	fn try_match(&self, next: char, input: &mut Cursor, _errors: &mut ErrorList) -> Option<Token> {
		match next {
			'#' => {
				let (multi, mut level) = if input.read_if('(') {
					(true, 1)
				} else {
					(false, 0)
				};

				let mut pos;
				let putback = loop {
					pos = input.clone();
					match input.read() {
						Some('\n' | '\r') if !multi => break true,
						Some('(') if multi => {
							level += 1;
						}
						Some(')') if multi => {
							level -= 1;
							if level == 0 {
								break false;
							}
						}
						Some(_) => {}
						None => break false,
					}
				};
				if putback {
					*input = pos;
				}
				Some(Comment::token(()))
			}

			_ => None,
		}
	}
}
