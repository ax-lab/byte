mod context;
mod cursor;
mod lex;
mod range;
mod token;

use once_cell::unsync::Lazy;

pub use super::input::*;

pub use context::*;
pub use cursor::*;
pub use lex::*;
pub use range::*;
pub use token::*;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Indent(pub usize);

pub enum LexerResult {
	None,
	Token(Token, Indent),
	Error(String),
}

#[derive(Debug)]
pub enum MatcherResult {
	None,
	Skip,
	Comment,
	Token(Token),
	Error(String),
}

pub trait Matcher {
	/// Tries to read the next recognized token from the input.
	///
	/// Returns [`LexerResult::None`] if the next token is not recognized or
	/// at the end of input.
	///
	/// The input will advance to the end of the recognized token iff the
	/// token is recognized.
	fn try_match(&self, next: char, input: &mut Cursor) -> MatcherResult;
}

mod lex_comment;
mod lex_identifier;
mod lex_line_break;
mod lex_number;
mod lex_space;
mod lex_string;
mod lex_symbol;

/// This is used for the lexer to determined what is a whitespace character.
pub fn is_space(char: char) -> bool {
	matches!(char, ' ' | '\t')
}

pub fn read_token<'a>(input: &mut Cursor<'a>) -> (LexerResult, Range<'a>) {
	let lexers = Lazy::new(|| {
		let lexers: Vec<Box<dyn Matcher>> = vec![
			Box::new(lex_space::LexSpace),
			Box::new(lex_comment::LexComment),
			Box::new(lex_line_break::LexLineBreak(Token::Break)),
			Box::new(lex_identifier::LexIdentifier(Token::Identifier)),
			Box::new(lex_string::LexLiteral(Token::Literal)),
			Box::new(lex_number::LexNumber(|n| Token::Integer(n))),
			Box::new(symbols()),
		];
		lexers
	});

	let mut pos = *input;
	let mut indent = Indent(pos.indent);
	let result = 'main: loop {
		if let Some(next) = input.read() {
			let start = *input;
			let mut skipped = false;
			for it in lexers.iter() {
				*input = start;
				match it.try_match(next, input) {
					MatcherResult::None => continue,
					MatcherResult::Error(error) => break 'main LexerResult::Error(error),
					next @ (MatcherResult::Skip | MatcherResult::Comment) => {
						pos = *input;
						if let MatcherResult::Skip = next {
							indent = Indent(pos.indent);
						}
						skipped = true;
						break;
					}
					MatcherResult::Token(token) => {
						break 'main LexerResult::Token(token, indent);
					}
				}
			}
			if !skipped {
				break LexerResult::Error(format!("invalid token"));
			}
		} else {
			break LexerResult::None;
		}
	};

	(result, Range { pos, end: *input })
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
