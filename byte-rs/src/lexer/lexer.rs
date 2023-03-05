use super::{Reader, Token};

pub enum LexerResult {
	None,
	Token(Token),
	Error(String),
}

pub trait Lexer: Sized {
	/// Tries to read the next recognized token from the input.
	///
	/// Returns [`LexerResult::None`] if the next token is not recognized or
	/// at the end of input.
	///
	/// The input will advance to the end of the recognized token iff the
	/// token is recognized.
	fn read(&self, next: char, input: &mut Reader) -> LexerResult;

	/// Creates a chained composite lexer.
	fn or<B: Lexer>(self, other: B) -> LexOr<Self, B> {
		LexOr { a: self, b: other }
	}
}

pub struct LexOr<A: Lexer, B: Lexer> {
	a: A,
	b: B,
}

impl<A: Lexer, B: Lexer> Lexer for LexOr<A, B> {
	fn read(&self, next: char, input: &mut Reader) -> LexerResult {
		let res = self.a.read(next, input);
		if let LexerResult::None = res {
			self.b.read(next, input)
		} else {
			res
		}
	}
}
