use std::fmt::{Debug, Display};

use crate::core::any::*;
use crate::core::context::*;
use crate::core::error::*;
use crate::core::input::*;

pub mod symbol;

pub trait Scanner: 'static {
	fn scan(&self, next: char, input: &mut Cursor, errors: &mut ErrorList) -> Option<TokenData>;
}

#[derive(Copy, Clone)]
pub struct TokenData {
	value: Value,
}

impl TokenData {
	pub fn is<T: IsToken>(&self) -> bool {
		self.value.is::<T>()
	}

	pub fn get<T: IsToken>(&self) -> Option<&'static T::Value> {
		self.value.get::<T>()
	}

	pub fn val<T: IsToken>(&self) -> Option<T::Value>
	where
		<T as IsToken>::Value: Copy,
	{
		self.value.val::<T>()
	}
}

pub trait IsToken: 'static + Sized {
	type Value: 'static;

	fn data(ctx: Context, value: Self::Value) -> TokenData {
		let value = <Self as Valued>::new(ctx, value);
		TokenData { value }
	}
}

impl<T: IsToken> Valued for T {
	type Value = T::Value;
}

#[derive(Copy, Clone)]
pub struct Token {
	data: TokenData,
	span: Span,
}

impl Token {
	pub fn invalid() -> Self {
		todo!()
	}

	pub fn skip() -> Self {
		todo!()
	}
}

pub struct Lexer {
	scanners: Vec<Box<dyn Scanner>>,
}

impl Lexer {
	pub fn new() -> Lexer {
		Lexer {
			scanners: Vec::new(),
		}
	}

	pub fn add_scanner<T: Scanner>(&mut self, scanner: T) {
		let scanner: Box<dyn Scanner> = Box::new(scanner);
		self.scanners.push(scanner);
	}

	pub fn add_symbol(&mut self, symbol: &str, data: TokenData) {
		todo!()
	}

	pub fn read(&self, input: &mut Cursor, errors: &mut ErrorList) -> Token {
		let sta = *input;
		let data = self.read_next(input, errors);
		let span = Span { sta, end: *input };
		assert!(input.offset() > sta.offset());
		Token { data, span }
	}

	fn read_next(&self, input: &mut Cursor, errors: &mut ErrorList) -> TokenData {
		todo!()
	}
}
