use std::collections::HashMap;

use super::{Input, LexResult, LexValue, Lexer, Reader};

pub struct SymbolTable<T: LexValue> {
	states: Vec<Entry<T>>,
}

impl<T: LexValue> Default for SymbolTable<T> {
	fn default() -> Self {
		SymbolTable {
			states: Default::default(),
		}
	}
}

struct Entry<T: LexValue> {
	value: Option<T>,
	next: HashMap<char, usize>,
}

impl<T: LexValue> SymbolTable<T> {
	pub fn add(&mut self, symbol: &'static str, value: T) {
		assert!(symbol.len() > 0);
		if self.states.len() == 0 {
			self.states.push(Entry {
				value: None,
				next: Default::default(),
			});
		}

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

impl<T: LexValue> Lexer<T> for SymbolTable<T> {
	fn read<S: Input>(&self, next: char, input: &mut Reader<S>) -> LexResult<T> {
		let state = self.get_next(0, next);
		let (mut state, valid) = if let Some((state, valid)) = state {
			(state, valid)
		} else {
			return LexResult::None;
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
			LexResult::Ok(self.states[index].value.unwrap())
		} else {
			LexResult::Error("invalid symbol".into())
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::lexer::Span;

	#[derive(Copy, Clone, Debug, Eq, PartialEq)]
	struct Token(pub &'static str);

	impl LexValue for Token {}

	#[test]
	fn lexer_should_parse_symbols() {
		let mut symbols = SymbolTable::default();
		symbols.add("+", Token("+"));
		symbols.add("++", Token("++"));
		symbols.add(".", Token("."));
		symbols.add("..", Token(".."));
		symbols.add("...", Token("..."));
		symbols.add(">", Token(">"));
		symbols.add(">>>>", Token("arrow"));

		check_symbols(&symbols, "", &[]);
		check_symbols(&symbols, "+", &[Token("+")]);
		check_symbols(&symbols, "++", &[Token("++")]);
		check_symbols(&symbols, "+++", &[Token("++"), Token("+")]);
		check_symbols(&symbols, ".", &[Token(".")]);
		check_symbols(&symbols, "..", &[Token("..")]);
		check_symbols(&symbols, "...", &[Token("...")]);
		check_symbols(&symbols, "....", &[Token("..."), Token(".")]);
		check_symbols(&symbols, ".....+", &[Token("..."), Token(".."), Token("+")]);
		check_symbols(&symbols, ">>>", &[Token(">"), Token(">"), Token(">")]);
		check_symbols(&symbols, ">>>>", &[Token("arrow")]);
		check_symbols(&symbols, ">>>>>>>>", &[Token("arrow"), Token("arrow")]);
	}

	fn check_symbols<T: LexValue + Eq + std::fmt::Debug>(
		symbols: &SymbolTable<T>,
		input: &'static str,
		expected: &[T],
	) {
		use crate::lexer::tests::TestInput;
		let mut input = Reader::from(TestInput::new(input));
		for (i, expected) in expected.iter().cloned().enumerate() {
			let next = input.read().expect("unexpected end of input");
			let pos = input.pos();
			let next = symbols.read(next, &mut input);
			let text = input.read_text(Span {
				pos,
				end: input.pos(),
			});
			match next {
				LexResult::Ok(actual) => assert_eq!(
					actual, expected,
					"unexpected symbol {:?} from `{}` at #{} (expected {:?})",
					actual, text, i, expected,
				),
				LexResult::Error(error) => {
					panic!("unexpected error at #{i}: {error} (consumed: `{text}`)")
				}
				LexResult::None => {
					panic!("expected token, got none at #{i} (consumed: `{text}`)")
				}
			}
		}

		assert!(input.read().is_none());
	}
}
