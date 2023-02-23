mod input;
mod span;
mod token;

pub use input::*;
use once_cell::unsync::Lazy;
pub use span::*;
pub use token::*;

mod lexer;
pub use lexer::*;

mod lex_comment;
mod lex_identifier;
mod lex_line_break;
mod lex_number;
mod lex_space;
mod lex_string;
mod lex_symbol;

pub fn read_token<T: Input>(input: &mut Reader<T>) -> (LexResult<Token>, Span) {
	let config = Lazy::new(|| {
		let space = lex_space::TokenSpace(());
		let skip = space;

		let comment = lex_comment::TokenComment(Token::Comment);
		let line_break = lex_line_break::TokenLineBreak(Token::LineBreak);
		let identifier = lex_identifier::TokenIdentifier(Token::Identifier);
		let string = lex_string::TokenString(Token::String);
		let number = lex_number::TokenNumber(Token::Integer);
		let symbol = symbols();
		let lexer = comment
			.or(line_break)
			.or(identifier)
			.or(string)
			.or(number)
			.or(symbol);
		(skip, lexer)
	});
	let (skip, lexer) = &*config;

	let mut pos = input.pos();
	let (res, pos) = loop {
		if let Some(next) = input.read() {
			match skip.read(next, input) {
				LexResult::Ok(..) => {
					pos = input.pos();
				}
				LexResult::None => {
					let res = lexer.read(next, input);
					break (res, pos);
				}
				LexResult::Error(error) => break (LexResult::Error(error), pos),
			}
		} else {
			break (LexResult::None, pos);
		}
	};

	let res = if let LexResult::None = res {
		if let Some(error) = input.error() {
			LexResult::Error(error)
		} else {
			LexResult::None
		}
	} else {
		res
	};

	(
		res,
		Span {
			pos,
			end: input.pos(),
		},
	)
}

fn symbols() -> lex_symbol::SymbolTable<Token> {
	let mut sym = lex_symbol::SymbolTable::default();
	sym.add(",", Token::Symbol(","));
	sym.add("+", Token::Symbol("+"));
	sym.add("-", Token::Symbol("-"));
	sym.add("*", Token::Symbol("*"));
	sym.add("/", Token::Symbol("/"));
	sym.add("%", Token::Symbol("%"));
	sym.add("=", Token::Symbol("="));
	sym.add("==", Token::Symbol("=="));
	sym.add("!", Token::Symbol("!"));
	sym.add("?", Token::Symbol("?"));
	sym.add(":", Token::Symbol(":"));
	sym.add("(", Token::Symbol("("));
	sym.add(")", Token::Symbol(")"));
	sym.add(".", Token::Symbol("."));
	sym.add("..", Token::Symbol(".."));
	sym
}

#[cfg(test)]
mod tests {
	use super::*;

	pub struct TestInput {
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
