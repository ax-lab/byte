use std::marker::PhantomData;

use super::{Input, Reader};

pub trait LexValue: Copy + std::fmt::Debug {}

impl LexValue for () {}

impl LexValue for &'static str {}

pub enum LexResult<T: LexValue> {
	Ok(T),
	None,
	Error(String),
}

pub trait Lexer<T: LexValue>: Sized {
	// IMPORTANT: LexResult::None never consumes characters unless it is the end of input!
	fn read<S: Input>(&self, next: char, input: &mut Reader<S>) -> LexResult<T>;

	fn or<B: Lexer<T>>(self, other: B) -> LexOr<Self, B, T> {
		LexOr {
			a: self,
			b: other,
			phantom: Default::default(),
		}
	}
}

pub struct LexOr<A: Lexer<T>, B: Lexer<T>, T: LexValue> {
	a: A,
	b: B,
	phantom: PhantomData<T>,
}

impl<A: Lexer<T>, B: Lexer<T>, T: LexValue> Lexer<T> for LexOr<A, B, T> {
	fn read<S: Input>(&self, next: char, input: &mut Reader<S>) -> LexResult<T> {
		let res = self.a.read(next, input);
		if let LexResult::None = res {
			self.b.read(next, input)
		} else {
			res
		}
	}
}
