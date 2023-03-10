use super::{Lexer, LexerResult, Reader, Span, Token};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct LexString(pub Span);

impl LexString {
	pub fn content_span(&self) -> Span {
		self.0
	}
}

pub struct LexLiteral<F: Fn(LexString) -> Token>(pub F);

impl<F: Fn(LexString) -> Token> Lexer for LexLiteral<F> {
	fn read(&self, next: char, input: &mut Reader) -> LexerResult {
		match next {
			'\'' => {
				let pos = input.pos();
				loop {
					let end = input.pos();
					match input.read() {
						Some('\'') => {
							let str = LexString(Span { pos, end });
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
