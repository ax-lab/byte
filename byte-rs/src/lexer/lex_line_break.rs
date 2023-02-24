use super::{Input, IsToken, Lexer, LexerResult, Reader};

pub struct LexLineBreak<T: IsToken>(pub T);

impl<T: IsToken> Lexer<T> for LexLineBreak<T> {
	fn read<S: Input>(&self, next: char, input: &mut Reader<S>) -> LexerResult<T> {
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
