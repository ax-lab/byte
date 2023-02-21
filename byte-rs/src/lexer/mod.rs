mod input;
mod span;
mod symbols;
mod token;

pub use input::*;
pub use span::*;
pub use symbols::*;
pub use token::*;

pub struct State {
	pub symbols: SymbolTable,
}

impl Default for State {
	fn default() -> Self {
		State {
			symbols: SymbolTable::new(),
		}
	}
}

pub fn read_token<I: Input>(input: &mut Reader<I>, state: &mut State) -> (Token, bool) {
	match input.read() {
		Some(',') => (Token::Comma, true),

		Some(' ' | '\t') => {
			let mut pos;
			loop {
				pos = input.save();
				match input.read() {
					Some(' ' | '\t') => {}
					_ => break,
				}
			}
			input.restore(pos);
			return (Token::None, true);
		}

		Some('#') => {
			let (multi, mut level) = if input.read_if('(') {
				(true, 1)
			} else {
				(false, 0)
			};

			let mut pos;
			let putback = loop {
				pos = input.save();
				match input.read() {
					Some('\n' | '\r') if !multi => break true,
					Some('(') if multi => {
						level += 1;
					}
					Some(')') if multi => {
						level -= 1;
						if level == 0 {
							break false;
						}
					}
					Some(_) => {}
					None => break false,
				}
			};
			if putback {
				input.restore(pos);
			}
			return (Token::Comment, true);
		}

		Some('\r') => {
			input.read_if('\n');
			(Token::LineBreak, true)
		}

		Some('\n') => (Token::LineBreak, true),

		Some('0'..='9') => {
			let mut pos;
			loop {
				pos = input.save();
				match input.read() {
					Some('0'..='9') => {}
					_ => {
						break;
					}
				}
			}
			input.restore(pos);
			(Token::Integer, true)
		}

		Some('a'..='z' | 'A'..='Z' | '_') => {
			let mut pos;
			loop {
				pos = input.save();
				match input.read() {
					Some('a'..='z' | 'A'..='Z' | '_' | '0'..='9') => {}
					_ => {
						break;
					}
				}
			}
			input.restore(pos);
			(Token::Identifier, true)
		}

		Some('\'') => loop {
			match input.read() {
				Some('\'') => {
					break (Token::String, true);
				}

				None => {
					break (Token::Symbol, true);
				}

				_ => {}
			}
		},

		None => {
			if let Some(err) = input.error() {
				panic!("input error: {err}");
			} else {
				(Token::None, false)
			}
		}

		Some(char) => {
			if let Some((mut current, valid)) = state.symbols.push(0, char) {
				let mut last_valid = if valid { Some(input.save()) } else { None };
				let mut pos = input.save();
				while let Some(next) = input.read() {
					if let Some((next, valid)) = state.symbols.push(current, next) {
						pos = input.save();
						current = next;
						if valid {
							last_valid = Some(pos);
						}
					} else {
						break;
					}
				}
				if let Some(valid) = last_valid {
					input.restore(valid);
					(Token::Symbol, true)
				} else {
					input.restore(pos);
					(Token::Invalid, true)
				}
			} else {
				(Token::Invalid, true)
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn lexer_should_parse_symbols() {
		let mut symbols = SymbolTable::new();
		symbols.add_symbol("+");
		symbols.add_symbol("++");
		symbols.add_symbol(".");
		symbols.add_symbol("..");
		symbols.add_symbol("...");
		symbols.add_symbol(">");
		symbols.add_symbol(">>>>");

		check_symbols(&symbols, "", &[]);
		check_symbols(&symbols, "+", &["+"]);
		check_symbols(&symbols, "++", &["++"]);
		check_symbols(&symbols, "+++", &["++", "+"]);
		check_symbols(&symbols, ".", &["."]);
		check_symbols(&symbols, "..", &[".."]);
		check_symbols(&symbols, "...", &["..."]);
		check_symbols(&symbols, "....", &["...", "."]);
		check_symbols(&symbols, ".....+", &["...", "..", "+"]);
		check_symbols(&symbols, ">>>", &[">", ">", ">"]);
		check_symbols(&symbols, ">>>>", &[">>>>"]);
		check_symbols(&symbols, ">>>>>>>>", &[">>>>", ">>>>"]);
	}

	#[test]
	fn symbol_table_should_recognize_added_symbols() {
		let mut x = SymbolTable::new();
		x.add_symbol("a");
		x.add_symbol("b");
		x.add_symbol("car");
		x.add_symbol("12");
		x.add_symbol("123");
		x.add_symbol("12345");

		let mut state;

		let mut check = move |state: &mut usize, char: char, expected: Option<bool>| {
			let actual = x.push(*state, char);
			if let Some((next, _)) = actual {
				*state = next;
			} else {
				*state = 0;
			}
			assert_eq!(expected, actual.map(|x| x.1));
		};

		state = 0;
		check(&mut state, 'a', Some(true));
		check(&mut state, 'x', None);

		state = 0;
		check(&mut state, 'b', Some(true));
		check(&mut state, 'b', None);

		state = 0;
		check(&mut state, 'c', Some(false));
		check(&mut state, 'a', Some(false));
		check(&mut state, 'r', Some(true));
		check(&mut state, 'x', None);

		state = 0;
		check(&mut state, '1', Some(false));
		check(&mut state, '2', Some(true));
		check(&mut state, '3', Some(true));
		check(&mut state, '4', Some(false));
		check(&mut state, '5', Some(true));
		check(&mut state, 'x', None);
	}

	fn check_symbols(symbols: &SymbolTable, input: &'static str, expected: &[&'static str]) {
		let mut input = Reader::from(TestInput::new(input));
		let mut state = State::default();
		state.symbols = symbols.clone();
		for (i, expected) in expected.iter().cloned().enumerate() {
			let pos = input.pos();
			let next = read_token(&mut input, &mut state);
			let text = input.read_text(Span {
				pos,
				end: input.pos(),
			});
			assert_eq!(
				next,
				(Token::Symbol, true),
				"unexpected output {:?} `{}` at #{} (expected `{}`)",
				next,
				text,
				i,
				expected,
			);
			assert_eq!(
				text, expected,
				"unexpected symbol `{}` at #{} (expected `{}`)",
				text, i, expected
			);
		}

		let next = read_token(&mut input, &mut state);
		assert_eq!(next, (Token::None, false));
	}

	struct TestInput {
		chars: Vec<char>,
		pos: usize,
		txt: String,
	}

	impl TestInput {
		pub fn new(input: &'static str) -> Self {
			TestInput {
				chars: input.chars().collect(),
				pos: 0,
				txt: String::new(),
			}
		}
	}

	impl Input for TestInput {
		type Error = String;

		fn read_text(&mut self, pos: usize, end: usize) -> &str {
			let chars = &self.chars[pos..end];
			self.txt = chars.into_iter().collect();
			return &self.txt;
		}

		fn offset(&self) -> usize {
			self.pos
		}

		fn set_offset(&mut self, pos: usize) {
			self.pos = pos;
		}

		fn read(&mut self) -> Option<char> {
			let offset = self.pos;
			if offset < self.chars.len() {
				self.pos += 1;
				Some(self.chars[offset])
			} else {
				None
			}
		}

		fn error(&self) -> Option<String> {
			None
		}
	}
}
