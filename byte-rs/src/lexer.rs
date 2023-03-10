mod lex;
mod reader;
mod source;
mod token;

use once_cell::unsync::Lazy;

pub use super::input::*;

pub use lex::*;
pub use reader::*;
pub use source::*;
pub use token::*;

pub enum LexerResult {
	None,
	Skip,
	Token(Token),
	Error(String),
}

pub trait Lexer: Sized {
	/// Tries to read the next recognized token from the input.
	///
	/// Returns [`LexerResult::None`] if the next token is not recognized or
	/// at the end of input.
	///
	/// The input will advance to the end of the recognized token iff the
	/// token is recognized.
	fn read(&self, next: char, input: &mut Reader) -> LexerResult;

	/// Creates a chained composite lexer.
	fn or<B: Lexer>(self, other: B) -> LexOr<Self, B> {
		LexOr { a: self, b: other }
	}
}

pub struct LexOr<A: Lexer, B: Lexer> {
	a: A,
	b: B,
}

impl<A: Lexer, B: Lexer> Lexer for LexOr<A, B> {
	fn read(&self, next: char, input: &mut Reader) -> LexerResult {
		let res = self.a.read(next, input);
		if let LexerResult::None = res {
			self.b.read(next, input)
		} else {
			res
		}
	}
}

mod lex_comment;
mod lex_identifier;
mod lex_line_break;
mod lex_number;
mod lex_space;
mod lex_string;
mod lex_symbol;

pub fn read_token(input: &mut Reader) -> (LexerResult, Span) {
	let config = Lazy::new(|| {
		let space = lex_space::LexSpace;
		let skip = space;

		let comment = lex_comment::LexComment;
		let line_break = lex_line_break::LexLineBreak(Token::Break);
		let identifier = lex_identifier::LexIdentifier(Token::Identifier);
		let string = lex_string::LexLiteral(Token::Literal);
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
				LexerResult::Token(..) | LexerResult::Skip => {
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

fn symbols() -> lex_symbol::LexSymbol {
	let mut sym = lex_symbol::LexSymbol::default();
	sym.add_symbol(",", Token::Symbol(","));
	sym.add_symbol(";", Token::Symbol(";"));
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
		txt: &'static str,
	}

	impl TestInput {
		pub fn new(input: &'static str) -> Self {
			TestInput { txt: input }
		}
	}

	impl Input for TestInput {
		fn len(&self) -> usize {
			self.txt.len()
		}

		fn read(&self, pos: usize, end: usize) -> &[u8] {
			&self.txt.as_bytes()[pos..end]
		}
	}
}
