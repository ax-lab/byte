use crate::input::*;

use std::{collections::HashMap, rc::Rc};

use crate::lexer::LexerError;

use super::{Matcher, MatcherResult, Token};

#[derive(Clone)]
pub struct SymbolTable {
	states: Vec<Entry>,
}

impl Default for SymbolTable {
	fn default() -> Self {
		let mut out = SymbolTable {
			states: Default::default(),
		};
		out.states.push(Entry {
			value: None,
			next: Default::default(),
		});
		out
	}
}

#[derive(Clone)]
struct Entry {
	value: Option<Token>,
	next: HashMap<char, usize>,
}

impl SymbolTable {
	pub fn add_symbol(&mut self, symbol: &'static str, value: Token) {
		assert!(symbol.len() > 0);
		let mut current = 0;
		for char in symbol.chars() {
			current = {
				let len = self.states.len();
				let state = &mut self.states[current];
				let next = state.next.entry(char);
				*next.or_insert(len)
			};
			if current == self.states.len() {
				self.states.push(Entry {
					value: None,
					next: Default::default(),
				});
			}
		}
		self.states[current].value = Some(value);
	}

	fn get_next(&self, current: usize, next: char) -> Option<(usize, bool)> {
		let state = &self.states[current];
		if let Some(&next) = state.next.get(&next) {
			let state = &self.states[next];
			let valid = state.value.is_some();
			Some((next, valid))
		} else {
			None
		}
	}
}

#[derive(Clone)]
pub struct MatchSymbol {
	symbols: Rc<SymbolTable>,
}

impl MatchSymbol {
	pub fn new(symbols: Rc<SymbolTable>) -> Self {
		MatchSymbol { symbols }
	}
}

impl Matcher for MatchSymbol {
	fn try_match(&self, next: char, input: &mut Cursor) -> MatcherResult {
		let symbols = &self.symbols;
		let state = symbols.get_next(0, next);
		let (mut state, valid) = if let Some((state, valid)) = state {
			(state, valid)
		} else {
			return MatcherResult::Error(LexerError::InvalidSymbol);
		};

		let mut last_pos = *input;
		let mut valid = if valid { Some((last_pos, state)) } else { None };

		while let Some(next) = input.read() {
			if let Some((next, is_valid)) = symbols.get_next(state, next) {
				(state, last_pos) = (next, *input);
				if is_valid {
					valid = Some((last_pos, state));
				}
			} else {
				break;
			}
		}
		if let Some((pos, index)) = valid {
			*input = pos;
			MatcherResult::Token(symbols.states[index].value.clone().unwrap())
		} else {
			MatcherResult::Error(LexerError::InvalidSymbol)
		}
	}

	fn clone_box(&self) -> Box<dyn Matcher> {
		Box::new(self.clone())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn lexer_should_parse_symbols() {
		let mut sym = SymbolTable::default();
		sym.add_symbol("+", Token::Symbol("+"));
		sym.add_symbol("++", Token::Symbol("++"));
		sym.add_symbol(".", Token::Symbol("."));
		sym.add_symbol("..", Token::Symbol(".."));
		sym.add_symbol("...", Token::Symbol("..."));
		sym.add_symbol(">", Token::Symbol(">"));
		sym.add_symbol(">>>>", Token::Symbol("arrow"));

		let sym = MatchSymbol::new(Rc::new(sym.into()));
		let sym = &sym;
		check_symbols(sym, "", &[]);
		check_symbols(sym, "+", &[Token::Symbol("+")]);
		check_symbols(sym, "++", &[Token::Symbol("++")]);
		check_symbols(sym, "+++", &[Token::Symbol("++"), Token::Symbol("+")]);
		check_symbols(sym, ".", &[Token::Symbol(".")]);
		check_symbols(sym, "..", &[Token::Symbol("..")]);
		check_symbols(sym, "...", &[Token::Symbol("...")]);
		check_symbols(sym, "....", &[Token::Symbol("..."), Token::Symbol(".")]);
		check_symbols(
			sym,
			".....+",
			&[
				Token::Symbol("..."),
				Token::Symbol(".."),
				Token::Symbol("+"),
			],
		);
		check_symbols(
			sym,
			">>>",
			&[Token::Symbol(">"), Token::Symbol(">"), Token::Symbol(">")],
		);
		check_symbols(sym, ">>>>", &[Token::Symbol("arrow")]);
		check_symbols(
			sym,
			">>>>>>>>",
			&[Token::Symbol("arrow"), Token::Symbol("arrow")],
		);
	}

	fn check_symbols(symbols: &MatchSymbol, input: &'static str, expected: &[Token]) {
		let input = open_str("literal", input);
		let mut input = input.sta();
		for (i, expected) in expected.iter().cloned().enumerate() {
			let next = input.read().expect("unexpected end of input");
			let pos = input;
			let next = symbols.try_match(next, &mut input);
			let end = input;
			let text = input.src().text(Span { sta: pos, end });
			match next {
				MatcherResult::Token(actual) => assert_eq!(
					actual, expected,
					"unexpected symbol {:?} from `{}` at #{} (expected {:?})",
					actual, text, i, expected,
				),
				MatcherResult::Error(error) => {
					panic!("unexpected error at #{i}: {error} (consumed: `{text}`)")
				}
				result => {
					panic!("expected token, got {result:?} at #{i} (consumed: `{text}`)")
				}
			}
		}

		assert!(input.read().is_none());
	}
}
