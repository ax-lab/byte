use std::marker::PhantomData;

use super::{Input, Reader};

pub enum LexerResult<T: IsToken> {
	None,
	Token(T),
	Error(String),
}

pub trait IsToken: Clone + std::fmt::Debug {}

impl IsToken for () {}

impl IsToken for &'static str {}

pub trait Lexer<T: IsToken>: Sized {
	/// Tries to read the next recognized token from the input.
	///
	/// Returns [`LexerResult::None`] if the next token is not recognized or
	/// at the end of input.
	///
	/// The input will advance to the end of the recognized token iff the
	/// token is recognized.
	fn read<S: Input>(&self, next: char, input: &mut Reader<S>) -> LexerResult<T>;

	/// Creates a chained composite lexer.
	fn or<B: Lexer<T>>(self, other: B) -> LexOr<Self, B, T> {
		LexOr {
			a: self,
			b: other,
			phantom: Default::default(),
		}
	}
}

pub struct LexOr<A: Lexer<T>, B: Lexer<T>, T: IsToken> {
	a: A,
	b: B,
	phantom: PhantomData<T>,
}

impl<A: Lexer<T>, B: Lexer<T>, T: IsToken> Lexer<T> for LexOr<A, B, T> {
	fn read<S: Input>(&self, next: char, input: &mut Reader<S>) -> LexerResult<T> {
		let res = self.a.read(next, input);
		if let LexerResult::None = res {
			self.b.read(next, input)
		} else {
			res
		}
	}
}
