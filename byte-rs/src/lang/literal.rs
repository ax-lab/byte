use crate::core::error::*;
use crate::core::input::*;
use crate::lexer::*;

pub struct Literal;

impl IsToken for Literal {
	type Value = String;

	fn name() -> &'static str {
		"Literal"
	}
}

#[derive(Debug)]
struct UnclosedLiteral;

impl IsError for UnclosedLiteral {
	fn output(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "unclosed string literal")
	}
}

impl Matcher for Literal {
	fn try_match(&self, next: char, input: &mut Cursor, errors: &mut ErrorList) -> Option<Token> {
		match next {
			'\'' => {
				let pos = input.clone();
				loop {
					let end = input.clone();
					match input.read() {
						Some('\'') => {
							let span = Span { sta: pos, end };
							let value = span.text().to_string();
							let token = Literal::token(value);
							break Some(token);
						}

						None => {
							let span = Span { sta: pos, end };
							errors.add(Error::new(span, UnclosedLiteral));
							break Some(Token::Invalid);
						}

						Some(_) => {}
					}
				}
			}

			_ => None,
		}
	}
}
