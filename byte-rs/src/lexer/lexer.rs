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

pub fn skip<A: Lexer<T1>, B: Lexer<T2>, T1: LexValue, T2: LexValue>(
	skip: A,
	next: B,
) -> impl Lexer<T2> {
	LexSkip {
		skip,
		next,
		phantom1: Default::default(),
		phantom2: Default::default(),
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

pub struct LexSkip<A: Lexer<T1>, B: Lexer<T2>, T1: LexValue, T2: LexValue> {
	skip: A,
	next: B,
	phantom1: PhantomData<T1>,
	phantom2: PhantomData<T2>,
}

impl<A: Lexer<T1>, B: Lexer<T2>, T1: LexValue, T2: LexValue> Lexer<T2> for LexSkip<A, B, T1, T2> {
	fn read<S: Input>(&self, next: char, input: &mut Reader<S>) -> LexResult<T2> {
		match self.skip.read(next, input) {
			LexResult::Error(err) => LexResult::Error(err),
			LexResult::None => self.next.read(next, input),
			LexResult::Ok(..) => {
				if let Some(next) = input.read() {
					self.next.read(next, input)
				} else {
					LexResult::None
				}
			}
		}
	}
}
