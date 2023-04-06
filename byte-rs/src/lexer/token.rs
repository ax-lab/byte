use std::any::TypeId;

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
	Value(TokenValueData),
}

#[derive(Clone, Debug)]
pub struct TokenValueData {
	token: TypeId,
	value: Value,
}

impl std::fmt::Display for TokenValueData {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.value)
	}
}

impl PartialEq for TokenValueData {
	fn eq(&self, other: &Self) -> bool {
		self.token == other.token && self.value == other.value
	}
}

impl Eq for TokenValueData {}

impl Token {
	pub fn is<T: TokenValue>(&self) -> bool {
		match self {
			Token::Value(value) => value.token == TypeId::of::<T>(),
			_ => false,
		}
	}

	pub fn get<T: TokenValue>(&self) -> Option<&T::Value> {
		match self {
			Token::Value(data) => {
				if data.token == TypeId::of::<T>() {
					data.value.get()
				} else {
					None
				}
			}
			_ => None,
		}
	}

	pub fn get_closing(&self) -> Option<&'static str> {
		match self {
			Token::Symbol("(") => Some(")"),
			_ => return None,
		}
	}
}

pub trait TokenValue: 'static + Sized {
	type Value: Clone + IsValue;

	fn token(value: Self::Value) -> Token {
		let value = Value::new(value);
		Token::Value(TokenValueData {
			token: TypeId::of::<Self>(),
			value,
		})
	}
}
