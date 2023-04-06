use crate::core::input::*;

use super::*;

pub struct Literal;

impl TokenValue for Literal {
	type Value = String;
}

#[derive(Debug)]
struct UnclosedLiteral;

impl ErrorInfo for UnclosedLiteral {
	fn output(&self, f: &mut std::fmt::Formatter<'_>, span: &Span) -> std::fmt::Result {
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
