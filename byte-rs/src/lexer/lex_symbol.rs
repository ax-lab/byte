use std::collections::HashMap;

use super::{IsToken, Lexer, LexerResult, Reader};

pub struct LexSymbol<T: IsToken> {
	states: Vec<Entry<T>>,
}

impl<T: IsToken> Default for LexSymbol<T> {
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

struct Entry<T: IsToken> {
	value: Option<T>,
	next: HashMap<char, usize>,
}

impl<T: IsToken> LexSymbol<T> {
	pub fn add_symbol(&mut self, symbol: &'static str, value: T) {
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

impl<T: IsToken> Lexer<T> for LexSymbol<T> {
	fn read(&self, next: char, input: &mut Reader) -> LexerResult<T> {
		let state = self.get_next(0, next);
		let (mut state, valid) = if let Some((state, valid)) = state {
			(state, valid)
		} else {
			return LexerResult::Error("invalid symbol".into());
		};

		let mut last_pos = input.save();
		let mut valid = if valid { Some((last_pos, state)) } else { None };

		while let Some(next) = input.read() {
			if let Some((next, is_valid)) = self.get_next(state, next) {
				(state, last_pos) = (next, input.save());
				if is_valid {
					valid = Some((last_pos, state));
				}
			} else {
				break;
			}
		}
		if let Some((pos, index)) = valid {
			input.restore(pos);
			LexerResult::Token(self.states[index].value.clone().unwrap())
		} else {
			LexerResult::Error("invalid symbol".into())
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[derive(Copy, Clone, Debug, Eq, PartialEq)]
	struct Token(pub &'static str);

	impl IsToken for Token {}

	#[test]
	fn lexer_should_parse_symbols() {
		let mut lexer = LexSymbol::default();
		lexer.add_symbol("+", Token("+"));
		lexer.add_symbol("++", Token("++"));
		lexer.add_symbol(".", Token("."));
		lexer.add_symbol("..", Token(".."));
		lexer.add_symbol("...", Token("..."));
		lexer.add_symbol(">", Token(">"));
		lexer.add_symbol(">>>>", Token("arrow"));

		check_symbols(&lexer, "", &[]);
		check_symbols(&lexer, "+", &[Token("+")]);
		check_symbols(&lexer, "++", &[Token("++")]);
		check_symbols(&lexer, "+++", &[Token("++"), Token("+")]);
		check_symbols(&lexer, ".", &[Token(".")]);
		check_symbols(&lexer, "..", &[Token("..")]);
		check_symbols(&lexer, "...", &[Token("...")]);
		check_symbols(&lexer, "....", &[Token("..."), Token(".")]);
		check_symbols(&lexer, ".....+", &[Token("..."), Token(".."), Token("+")]);
		check_symbols(&lexer, ">>>", &[Token(">"), Token(">"), Token(">")]);
		check_symbols(&lexer, ">>>>", &[Token("arrow")]);
		check_symbols(&lexer, ">>>>>>>>", &[Token("arrow"), Token("arrow")]);
	}

	fn check_symbols<T: IsToken + Eq + std::fmt::Debug>(
		symbols: &LexSymbol<T>,
		input: &'static str,
		expected: &[T],
	) {
		use crate::lexer::tests::TestInput;
		let mut input = Reader::from(TestInput::new(input));
		for (i, expected) in expected.iter().cloned().enumerate() {
			let next = input.read().expect("unexpected end of input");
			let pos = input.pos();
			let next = symbols.read(next, &mut input);
			let end = input.pos();
			let input = input.inner();
			let text = input.read_text(pos.offset, end.offset);
			match next {
				LexerResult::Token(actual) => assert_eq!(
					actual, expected,
					"unexpected symbol {:?} from `{}` at #{} (expected {:?})",
					actual, text, i, expected,
				),
				LexerResult::Error(error) => {
					panic!("unexpected error at #{i}: {error} (consumed: `{text}`)")
				}
				LexerResult::None => {
					panic!("expected token, got none at #{i} (consumed: `{text}`)")
				}
			}
		}

		assert!(input.read().is_none());
	}
}
