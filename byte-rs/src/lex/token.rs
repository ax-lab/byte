use crate::core::any::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Token {
	None,
	Invalid,
	Break,
	Indent,
	Dedent,
	Identifier,
	Symbol(&'static str),
	Value(Value),
}

impl Token {
	pub fn is<T: TokenValue>(&self) -> bool {
		match self {
			Token::Value(value) => value.is::<T>(),
			_ => false,
		}
	}

	pub fn get<T: TokenValue>(&self) -> Option<T::Value> {
		match self {
			Token::Value(value) => value.get::<T>().map(|x| x.clone()),
			_ => None,
		}
	}
}

pub trait TokenValue: 'static + Sized {
	type Value: Clone + 'static;

	fn token(value: Self::Value) -> Token {
		let value = <Self as Valued>::new(value);
		Token::Value(value)
	}
}

impl<T: TokenValue> Valued for T {
	type Value = T::Value;
}
