use std::rc::Rc;

use crate::core::error::*;
use crate::core::input::*;

use super::*;

/// Provides low-level token scanning using a configurable list of matchers
/// and symbol table.
///
/// This is used by the [`Lexer`].
#[derive(Clone)]
pub struct Scanner {
	scanners: Vec<Rc<dyn Matcher>>,
	symbols: SymbolTable,
}

impl Scanner {
	pub fn new() -> Scanner {
		Scanner {
			scanners: Vec::new(),
			symbols: SymbolTable::default(),
		}
	}

	pub fn add_matcher<T: Matcher + 'static>(&mut self, scanner: T) {
		self.scanners.push(Rc::new(scanner));
	}

	pub fn add_symbol(&mut self, symbol: &'static str, value: Token) {
		self.symbols.add_symbol(symbol, value);
	}

	/// Skip spaces and empty lines from the input cursor.
	pub fn skip(&self, input: &mut Cursor) {
		let is_start = input.col() == 0;
		let mut saved = input.clone();
		while let Some(next) = input.read() {
			if is_space(next) || (is_start && next == '\n') {
				saved = input.clone();
			} else {
				break;
			}
		}
		*input = saved;
	}

	pub fn read(&self, input: &mut Cursor, errors: &mut ErrorList) -> Token {
		let mut start = input.clone();
		if let Some(next) = input.read() {
			if next == '\n' {
				return Token::Break;
			}
			let saved = (input.clone(), errors.clone());
			for scanner in self.scanners.iter() {
				if let Some(token) = scanner.try_match(next, input, errors) {
					return token;
				} else {
					(*input, *errors) = saved.clone();
				}
			}

			// if none of the scanners matched, try the symbols
			if let Some(token) = self.symbols.try_match(next, input, errors) {
				token
			} else {
				(*input, *errors) = saved;
				errors.add(Error::new(
					Span {
						sta: start,
						end: input.clone(),
					},
					LexerError::InvalidSymbol,
				));
				Token::Invalid
			}
		} else {
			Token::None
		}
	}
}
