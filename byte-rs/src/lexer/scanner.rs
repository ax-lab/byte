use std::sync::Arc;

use crate::core::error::*;
use crate::core::input::*;

use super::*;

/// Implements raw token scanning using a configurable list of matchers
/// and symbol table.
///
/// This sits at a lower level than the [`Lexer`], scanning the physical
/// tokens in the input stream, without applying additional rules such as
/// indentation, skipping comments and empty lines.
#[derive(Clone)]
pub struct Scanner {
	scanners: Vec<Arc<dyn Matcher>>,
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
		self.scanners.push(Arc::new(scanner));
	}

	pub fn add_symbol(&mut self, symbol: &'static str, value: Token) {
		self.symbols.add_symbol(symbol, value);
	}

	/// Skip spaces and blank lines from the input cursor. This positions the
	/// input stream at the next raw token.
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
		let start = input.clone();
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
				errors.add_error(Error::new(LexerError::InvalidSymbol).at(Span {
					sta: start,
					end: input.clone(),
				}));
				Token::Invalid
			}
		} else {
			Token::None
		}
	}
}
