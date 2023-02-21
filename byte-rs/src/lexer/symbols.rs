use std::collections::HashMap;

#[derive(Clone)]
pub struct SymbolTable {
	states: Vec<(bool, HashMap<char, usize>)>,
}

impl SymbolTable {
	pub fn new() -> Self {
		SymbolTable {
			states: {
				let mut vec = Vec::new();
				vec.push(Default::default());
				vec
			},
		}
	}

	pub fn push(&mut self, current: usize, next: char) -> Option<(usize, bool)> {
		let (_, transitions) = &self.states[current];
		if let Some(&next) = transitions.get(&next) {
			let valid = self.states[next].0;
			Some((next, valid))
		} else {
			None
		}
	}

	pub fn add_symbol<S: AsRef<str>>(&mut self, symbol: S) {
		let symbol = symbol.as_ref();
		assert!(symbol.len() > 0);

		let mut current = 0;
		for char in symbol.chars() {
			current = {
				let len = self.states.len();
				let state = &mut self.states[current];
				let next = state.1.entry(char);
				*next.or_insert(len)
			};
			if current == self.states.len() {
				self.states.push(Default::default());
			}
		}
		self.states[current].0 = true;
	}
}
