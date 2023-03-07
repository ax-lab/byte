mod reader;
mod token;
mod token_stream;

use once_cell::unsync::Lazy;

pub use super::input::*;

pub use reader::*;
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

pub fn read_token(input: &mut Reader) -> (LexerResult, Span) {
	let config = Lazy::new(|| {
		let space = lex_space::LexSpace(Token::None);
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
