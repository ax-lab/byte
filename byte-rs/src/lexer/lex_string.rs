use super::{Cursor, Lexer, LexerResult, Token};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct LexString {
	pub pos: usize,
	pub end: usize,
}

pub struct LexLiteral<F: Fn(LexString) -> Token>(pub F);

impl<F: Fn(LexString) -> Token> Lexer for LexLiteral<F> {
	fn read(&self, next: char, input: &mut Cursor) -> LexerResult {
		match next {
			'\'' => {
				let pos = input.offset;
				loop {
					let end = input.offset;
					match input.read() {
						Some('\'') => {
							let str = LexString { pos, end };
							break LexerResult::Token(self.0(str));
						}

						None => break LexerResult::Error("unclosed string literal".into()),

						Some(_) => {}
					}
				}
			}

			_ => LexerResult::None,
		}
	}
}
