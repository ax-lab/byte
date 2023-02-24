mod input;
mod span;
mod token;
mod token_stream;

use once_cell::unsync::Lazy;

pub use input::*;
pub use span::*;
pub use token::*;
pub use token_stream::*;

mod lexer;
pub use lexer::*;

mod lex_comment;
mod lex_identifier;
mod lex_line_break;
mod lex_number;
mod lex_space;
mod lex_string;
mod lex_symbol;

pub fn read_token<T: Input>(input: &mut Reader<T>) -> (LexerResult<Token>, Span) {
	let config = Lazy::new(|| {
		let space = lex_space::LexSpace(());
		let skip = space;

		let comment = lex_comment::LexComment(Token::Comment);
		let line_break = lex_line_break::LexLineBreak(Token::LineBreak);
		let identifier = lex_identifier::LexIdentifier(|s| Token::Identifier(s));
		let string = lex_string::LexString(|s| Token::Literal(s));
		let number = lex_number::LexNumber(|n| Token::Integer(n));
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
				LexerResult::Token(..) => {
					pos = input.pos();
				}
				LexerResult::None => {
					let res = lexer.read(next, input);
					break (res, pos);
				}
				LexerResult::Error(error) => break (LexerResult::Error(error), pos),
			}
		} else {
			break (LexerResult::None, pos);
		}
	};

	(
		res,
		Span {
			pos,
			end: input.pos(),
		},
	)
}

fn symbols() -> lex_symbol::LexSymbol<Token> {
	let mut sym = lex_symbol::LexSymbol::default();
	sym.add_symbol(",", Token::Symbol(","));
	sym.add_symbol("+", Token::Symbol("+"));
	sym.add_symbol("-", Token::Symbol("-"));
	sym.add_symbol("*", Token::Symbol("*"));
	sym.add_symbol("/", Token::Symbol("/"));
	sym.add_symbol("%", Token::Symbol("%"));
	sym.add_symbol("=", Token::Symbol("="));
	sym.add_symbol("==", Token::Symbol("=="));
	sym.add_symbol("!", Token::Symbol("!"));
	sym.add_symbol("?", Token::Symbol("?"));
	sym.add_symbol(":", Token::Symbol(":"));
	sym.add_symbol("(", Token::Symbol("("));
	sym.add_symbol(")", Token::Symbol(")"));
	sym.add_symbol(".", Token::Symbol("."));
	sym.add_symbol("..", Token::Symbol(".."));
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
	}
}
