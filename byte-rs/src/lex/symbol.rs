use std::{collections::HashMap, rc::Rc};

use super::*;
use crate::core::input::*;

pub struct Symbol(&'static str);

impl Symbol {
	pub fn from(ctx: Context, sym: &'static str) -> TokenData {
		Symbol::data(ctx, Symbol(sym))
	}
}

impl IsToken for Symbol {
	type Value = Self;
}

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
	value: Option<TokenData>,
	next: HashMap<char, usize>,
}

impl SymbolTable {
	pub fn add_symbol(&mut self, symbol: &'static str, value: TokenData) {
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

impl Scanner for SymbolTable {
	fn scan(&self, next: char, input: &mut Cursor, errors: &mut ErrorList) -> Option<TokenData> {
		let state = self.get_next(0, next);
		let (mut state, valid) = if let Some((state, valid)) = state {
			(state, valid)
		} else {
			return None;
		};

		let mut last_pos = input.clone();
		let mut valid = if valid { Some((last_pos, state)) } else { None };

		while let Some(next) = input.read() {
			if let Some((next, is_valid)) = self.get_next(state, next) {
				(state, last_pos) = (next, input.clone());
				if is_valid {
					valid = Some((last_pos, state));
				}
			} else {
				break;
			}
		}
		if let Some((pos, index)) = valid {
			*input = pos;
			Some(self.states[index].value.clone().unwrap())
		} else {
			None
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::core::context::*;

	#[test]
	fn lexer_should_parse_symbols() {
		let ctx = Context::new();
		let mut sym = SymbolTable::default();
		sym.add_symbol("+", Symbol::from(ctx, "+"));
		sym.add_symbol("++", Symbol::from(ctx, "++"));
		sym.add_symbol(".", Symbol::from(ctx, "."));
		sym.add_symbol("..", Symbol::from(ctx, ".."));
		sym.add_symbol("...", Symbol::from(ctx, "..."));
		sym.add_symbol(">", Symbol::from(ctx, ">"));
		sym.add_symbol(">>>>", Symbol::from(ctx, "arrow"));

		let sym = &sym;
		check_symbols(ctx, sym, "", &[]);
		check_symbols(ctx, sym, "+", &["+"]);
		check_symbols(ctx, sym, "++", &["++"]);
		check_symbols(ctx, sym, "+++", &["++", "+"]);
		check_symbols(ctx, sym, ".", &["."]);
		check_symbols(ctx, sym, "..", &[".."]);
		check_symbols(ctx, sym, "...", &["..."]);
		check_symbols(ctx, sym, "....", &["...", "."]);
		check_symbols(ctx, sym, ".....+", &["...", "..", "+"]);
		check_symbols(ctx, sym, ">>>", &[">", ">", ">"]);
		check_symbols(ctx, sym, ">>>>", &["arrow"]);
		check_symbols(ctx, sym, ">>>>>>>>", &["arrow", "arrow"]);
	}

	fn check_symbols(
		ctx: Context,
		symbols: &SymbolTable,
		input: &'static str,
		expected: &[&'static str],
	) {
		let mut errors = ErrorList::new(ctx);
		let ctx = Context::new();
		let input = ctx.open_str("literal", input);
		let mut input = input.sta();
		for (i, expected) in expected.iter().cloned().enumerate() {
			let pos = input.clone();
			let char = input.read().expect("unexpected end of input");
			let next = symbols.scan(char, &mut input, &mut errors);
			let end = input.clone();

			assert!(errors.empty());
			let text = input.src().text(Span { sta: pos, end });
			let next = if let Some(next) = next {
				next
			} else {
				panic!("got no match at #{i}: (consumed: `{text}`)");
			};

			if let Some(Symbol(actual)) = next.get::<Symbol>() {
				assert_eq!(
					*actual, expected,
					"unexpected symbol {:?} from `{}` at #{} (expected {:?})",
					actual, text, i, expected,
				);
			} else {
				panic!("got invalid data at #{i}: (consumed: `{text}`)");
			}
		}

		assert!(input.read().is_none());
	}
}
