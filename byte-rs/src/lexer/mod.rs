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

pub fn read_token<I: Input>(input: &mut I, state: &mut State) -> (Token, bool) {
	match input.read() {
		Some(',') => (Token::Comma, true),

		Some(' ' | '\t') => {
			let mut pos;
			loop {
				pos = input.pos();
				match input.read() {
					Some(' ' | '\t') => {}
					_ => break,
				}
			}
			input.rewind(pos);
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
				pos = input.pos();
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
				input.rewind(pos);
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
				pos = input.pos();
				match input.read() {
					Some('0'..='9') => {}
					_ => {
						break;
					}
				}
			}
			input.rewind(pos);
			(Token::Integer, true)
		}

		Some('a'..='z' | 'A'..='Z' | '_') => {
			let mut pos;
			loop {
				pos = input.pos();
				match input.read() {
					Some('a'..='z' | 'A'..='Z' | '_' | '0'..='9') => {}
					_ => {
						break;
					}
				}
			}
			input.rewind(pos);
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
				let mut last_valid = if valid { Some(input.pos()) } else { None };
				let mut pos = input.pos();
				while let Some(next) = input.read() {
					if let Some((next, valid)) = state.symbols.push(current, next) {
						pos = input.pos();
						current = next;
						if valid {
							last_valid = Some(pos);
						}
					} else {
						break;
					}
				}
				if let Some(valid) = last_valid {
					input.rewind(valid);
					(Token::Symbol, true)
				} else {
					input.rewind(pos);
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
		let mut input = TestInput::new(input);
		let mut state = State::default();
		state.symbols = symbols.clone();
		for (i, expected) in expected.iter().cloned().enumerate() {
			let next = read_token(&mut input, &mut state);
			let text = input.text();
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
				text.as_str(),
				expected,
				"unexpected symbol `{}` at #{} (expected `{}`)",
				text,
				i,
				expected
			);
		}

		let next = read_token(&mut input, &mut state);
		assert_eq!(next, (Token::None, false));
	}

	struct TestInput {
		chars: Vec<char>,
		pos: Pos,
		txt: usize,
	}

	impl TestInput {
		pub fn new(input: &'static str) -> Self {
			TestInput {
				chars: input.chars().collect(),
				pos: Pos::default(),
				txt: 0,
			}
		}

		pub fn text(&mut self) -> String {
			let chars = &self.chars[self.txt..self.pos.offset];
			self.txt = self.pos.offset;
			chars.into_iter().collect()
		}
	}

	impl Input for TestInput {
		fn read_text(&mut self, _span: Span) -> &str {
			unimplemented!()
		}

		fn pos(&self) -> Pos {
			self.pos
		}

		fn read(&mut self) -> Option<char> {
			let offset = self.pos.offset;
			if offset < self.chars.len() {
				self.pos.offset += 1;
				Some(self.chars[offset])
			} else {
				None
			}
		}

		fn rewind(&mut self, pos: Pos) {
			self.pos = pos;
		}

		fn error(&self) -> Option<std::io::Error> {
			None
		}
	}
}
