use std::collections::HashMap;

use super::{Cursor, Lexer, LexerResult, Token};

pub struct LexSymbol {
	states: Vec<Entry>,
}

impl Default for LexSymbol {
	fn default() -> Self {
		let mut out = LexSymbol {
			states: Default::default(),
		};
		out.states.push(Entry {
			value: None,
			next: Default::default(),
		});
		out
	}
}

struct Entry {
	value: Option<Token>,
	next: HashMap<char, usize>,
}

impl LexSymbol {
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

impl Lexer for LexSymbol {
	fn read(&self, next: char, input: &mut Cursor) -> LexerResult {
		let state = self.get_next(0, next);
		let (mut state, valid) = if let Some((state, valid)) = state {
			(state, valid)
		} else {
			return LexerResult::Error("invalid symbol".into());
		};

		let mut last_pos = *input;
		let mut valid = if valid { Some((last_pos, state)) } else { None };

		while let Some(next) = input.read() {
			if let Some((next, is_valid)) = self.get_next(state, next) {
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
			LexerResult::Token(self.states[index].value.clone().unwrap())
		} else {
			LexerResult::Error("invalid symbol".into())
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn lexer_should_parse_symbols() {
		let mut lexer = LexSymbol::default();
		lexer.add_symbol("+", Token::Symbol("+"));
		lexer.add_symbol("++", Token::Symbol("++"));
		lexer.add_symbol(".", Token::Symbol("."));
		lexer.add_symbol("..", Token::Symbol(".."));
		lexer.add_symbol("...", Token::Symbol("..."));
		lexer.add_symbol(">", Token::Symbol(">"));
		lexer.add_symbol(">>>>", Token::Symbol("arrow"));

		check_symbols(&lexer, "", &[]);
		check_symbols(&lexer, "+", &[Token::Symbol("+")]);
		check_symbols(&lexer, "++", &[Token::Symbol("++")]);
		check_symbols(&lexer, "+++", &[Token::Symbol("++"), Token::Symbol("+")]);
		check_symbols(&lexer, ".", &[Token::Symbol(".")]);
		check_symbols(&lexer, "..", &[Token::Symbol("..")]);
		check_symbols(&lexer, "...", &[Token::Symbol("...")]);
		check_symbols(&lexer, "....", &[Token::Symbol("..."), Token::Symbol(".")]);
		check_symbols(
			&lexer,
			".....+",
			&[
				Token::Symbol("..."),
				Token::Symbol(".."),
				Token::Symbol("+"),
			],
		);
		check_symbols(
			&lexer,
			">>>",
			&[Token::Symbol(">"), Token::Symbol(">"), Token::Symbol(">")],
		);
		check_symbols(&lexer, ">>>>", &[Token::Symbol("arrow")]);
		check_symbols(
			&lexer,
			">>>>>>>>",
			&[Token::Symbol("arrow"), Token::Symbol("arrow")],
		);
	}

	fn check_symbols(symbols: &LexSymbol, input: &'static str, expected: &[Token]) {
		use crate::lexer::tests::TestInput;
		let input = TestInput::new(input);
		let mut input = Cursor::new(&input);
		for (i, expected) in expected.iter().cloned().enumerate() {
			let next = input.read().expect("unexpected end of input");
			let pos = input.offset;
			let next = symbols.read(next, &mut input);
			let end = input.offset;
			let text = input.source.read_text(pos, end);
			match next {
				LexerResult::Token(actual) => assert_eq!(
					actual, expected,
					"unexpected symbol {:?} from `{}` at #{} (expected {:?})",
					actual, text, i, expected,
				),
				LexerResult::Error(error) => {
					panic!("unexpected error at #{i}: {error} (consumed: `{text}`)")
				}
				LexerResult::Skip => {
					panic!("expected token, got comment at #{i} (consumed: `{text}`)")
				}
				LexerResult::None => {
					panic!("expected token, got none at #{i} (consumed: `{text}`)")
				}
			}
		}

		assert!(input.read().is_none());
	}
}
