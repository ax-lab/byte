use std::collections::HashMap;

use super::*;

//====================================================================================================================//
// Symbol table
//====================================================================================================================//

/// Configurable symbol table implementing the [`Matcher`] trait.
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
	value: Option<Node>,
	next: HashMap<char, usize>,
}

impl SymbolTable {
	pub fn add_symbol<T: IsNode>(&mut self, symbol: &'static str, value: T) {
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
		self.states[current].value = Some(Node::from(value));
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

impl Matcher for SymbolTable {
	fn try_match(&self, cursor: &mut Cursor, _errors: &mut Errors) -> Option<Node> {
		let next = if let Some(next) = cursor.read() {
			next
		} else {
			return None;
		};
		let state = self.get_next(0, next);
		let (mut state, valid) = if let Some((state, valid)) = state {
			(state, valid)
		} else {
			return None;
		};

		let mut last_pos = cursor.clone();
		let mut valid = if valid { Some((last_pos, state)) } else { None };

		while let Some(next) = cursor.read() {
			if let Some((next, is_valid)) = self.get_next(state, next) {
				(state, last_pos) = (next, cursor.clone());
				if is_valid {
					valid = Some((last_pos, state));
				}
			} else {
				break;
			}
		}
		if let Some((pos, index)) = valid {
			*cursor = pos;
			Some(self.states[index].value.clone().unwrap())
		} else {
			None
		}
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

		let sym = &sym;
		check_symbols(sym, "", &[]);
		check_symbols(sym, "+", &["+"]);
		check_symbols(sym, "++", &["++"]);
		check_symbols(sym, "+++", &["++", "+"]);
		check_symbols(sym, ".", &["."]);
		check_symbols(sym, "..", &[".."]);
		check_symbols(sym, "...", &["..."]);
		check_symbols(sym, "....", &["...", "."]);
		check_symbols(sym, ".....+", &["...", "..", "+"]);
		check_symbols(sym, ">>>", &[">", ">", ">"]);
		check_symbols(sym, ">>>>", &["arrow"]);
		check_symbols(sym, ">>>>>>>>", &["arrow", "arrow"]);
	}

	fn check_symbols(symbols: &SymbolTable, input: &'static str, expected: &[&'static str]) {
		let mut errors = Errors::default();
		let input = Input::from(input);
		let mut cursor = input.cursor();
		for (i, expected) in expected.iter().cloned().enumerate() {
			let pos = cursor.clone();
			let next = symbols.try_match(&mut cursor, &mut errors);
			let end = cursor.clone();

			assert!(errors.empty());
			let text = Span::from(&pos, &end);
			let text = text.text();
			if let Some(Token::Symbol(actual)) = next.as_ref().unwrap().get::<Token>() {
				assert_eq!(
					*actual, expected,
					"unexpected symbol {:?} from `{}` at #{} (expected {:?})",
					actual, text, i, expected,
				);
			} else {
				panic!("got invalid token at #{i}: (consumed: `{text}`, got `{next:?}`)");
			}
		}

		assert!(cursor.read().is_none());
	}
}
