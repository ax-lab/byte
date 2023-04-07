use super::*;

pub trait IsBlock: Clone {
	type Value;

	fn parse<T: Scope>(scope: T) -> (T, Self::Value);
}
