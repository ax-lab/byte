mod context;
mod cursor;
mod lex;
mod span;
mod token;

pub mod matcher;
pub use matcher::{Matcher, MatcherResult};

use once_cell::unsync::Lazy;

pub use super::input::*;

pub use context::*;
pub use cursor::*;
pub use lex::*;
pub use span::*;
pub use token::*;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Indent(pub usize);

pub enum LexerResult {
	None,
	Token(Token, Indent),
	Error(String),
}

/// This is used for the lexer to determined what is a whitespace character.
pub fn is_space(char: char) -> bool {
	matches!(char, ' ' | '\t')
}

pub fn read_token<'a>(input: &mut Cursor<'a>) -> (LexerResult, Span<'a>) {
	let lexers = Lazy::new(|| {
		let lexers: Vec<Box<dyn Matcher>> = vec![
			Box::new(matcher::MatchSpace),
			Box::new(matcher::MatchComment),
			Box::new(matcher::MatchLineBreak(Token::Break)),
			Box::new(matcher::MatchIdentifier(Token::Identifier)),
			Box::new(matcher::MatchLiteral(|pos, end| Token::Literal(pos, end))),
			Box::new(matcher::MatchNumber(|n| Token::Integer(n))),
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

	(result, Span { pos, end: *input })
}

fn symbols() -> matcher::SymbolTable {
	let mut sym = matcher::SymbolTable::default();
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
