use super::{Cursor, Lexer, LexerResult, Token};

pub struct LexLineBreak(pub Token);

impl Lexer for LexLineBreak {
	fn read(&self, next: char, input: &mut Cursor) -> LexerResult {
		match next {
			'\r' => {
				input.read_if('\n');
				LexerResult::Token(self.0.clone())
			}

			'\n' => LexerResult::Token(self.0.clone()),

			_ => LexerResult::None,
		}
	}
}
