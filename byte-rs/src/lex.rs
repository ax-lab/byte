use std::fmt::{Debug, Display};

use crate::core::context::*;
use crate::core::error::*;
use crate::core::input::*;

mod symbol;
mod token;

pub use token::*;

use symbol::SymbolTable;

#[derive(Debug)]
pub enum LexerError {
	InvalidSymbol,
}

impl ErrorInfo for LexerError {}

impl Display for LexerError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			LexerError::InvalidSymbol => write!(f, "invalid symbol"),
		}
	}
}

pub trait Scanner {
	fn scan(&self, next: char, input: &mut Cursor, errors: &mut ErrorList) -> Option<Token>;
}

#[derive(Clone, Debug)]
pub struct Lex {
	data: Token,
	span: Span,
}

/// Provides the lexer configuration and the low-level token scanning for
/// the compiler.
pub struct Lexer {
	scanners: Vec<Box<dyn Scanner>>,
	symbols: SymbolTable,
}

impl Lexer {
	pub fn new() -> Lexer {
		Lexer {
			scanners: Vec::new(),
			symbols: SymbolTable::default(),
		}
	}

	pub fn add_scanner<T: Scanner + 'static>(&mut self, scanner: T) {
		let scanner: Box<dyn Scanner> = Box::new(scanner);
		self.scanners.push(scanner);
	}

	pub fn add_symbol(&mut self, symbol: &'static str, value: Token) {
		self.symbols.add_symbol(symbol, value);
	}

	pub fn read(&self, input: &mut Cursor, errors: &mut ErrorList) -> Lex {
		let sta = input.clone();
		let data = self.read_next(input, errors);
		let span = Span {
			sta: sta.clone(),
			end: input.clone(),
		};
		assert!(input.offset() > sta.offset());
		Lex { data, span }
	}

	fn read_next(&self, input: &mut Cursor, errors: &mut ErrorList) -> Token {
		let mut start = input.clone();
		while let Some(next) = input.read() {
			if is_space(next) {
				start = input.clone();
				continue;
			}

			let saved = (input.clone(), errors.clone());
			for scanner in self.scanners.iter() {
				if let Some(token) = scanner.scan(next, input, errors) {
					return token;
				} else {
					(*input, *errors) = saved.clone();
				}
			}

			// if none of the scanners matched, try the symbols
			return if let Some(token) = self.symbols.scan(next, input, errors) {
				token
			} else {
				(*input, *errors) = saved;
				errors.at(
					Span {
						sta: start,
						end: input.clone(),
					},
					LexerError::InvalidSymbol,
				);
				Token::Invalid
			};
		}
		Token::None
	}
}
