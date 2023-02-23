use super::{Input, Reader};

pub trait LexValue: Copy + std::fmt::Debug {}

impl LexValue for () {}

impl LexValue for &'static str {}

pub enum LexResult<T: LexValue> {
	Ok(T),
	None,
	Error(String),
}

pub trait Lexer {
	type Value: LexValue;

	fn read<S: Input>(&self, next: char, input: &mut Reader<S>) -> LexResult<Self::Value>;
}
