mod context;
mod cursor;
mod lex;
mod span;
mod token;

mod config;
pub use config::Config;

pub mod matcher;
pub use matcher::{Matcher, MatcherResult};

use crate::Input;

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

pub fn open(input: &dyn Input) -> Context {
	let mut cfg = Config::default();
	cfg.add_matcher(Box::new(matcher::MatchSpace));
	cfg.add_matcher(Box::new(matcher::MatchComment));
	cfg.add_matcher(Box::new(matcher::MatchLineBreak(Token::Break)));
	cfg.add_matcher(Box::new(matcher::MatchIdentifier(Token::Identifier)));
	cfg.add_matcher(Box::new(matcher::MatchLiteral));
	cfg.add_matcher(Box::new(matcher::MatchNumber));

	cfg.add_symbol(",", Token::Symbol(","));
	cfg.add_symbol(";", Token::Symbol(";"));
	cfg.add_symbol("+", Token::Symbol("+"));
	cfg.add_symbol("-", Token::Symbol("-"));
	cfg.add_symbol("*", Token::Symbol("*"));
	cfg.add_symbol("/", Token::Symbol("/"));
	cfg.add_symbol("%", Token::Symbol("%"));
	cfg.add_symbol("=", Token::Symbol("="));
	cfg.add_symbol("==", Token::Symbol("=="));
	cfg.add_symbol("!", Token::Symbol("!"));
	cfg.add_symbol("?", Token::Symbol("?"));
	cfg.add_symbol(":", Token::Symbol(":"));
	cfg.add_symbol("(", Token::Symbol("("));
	cfg.add_symbol(")", Token::Symbol(")"));
	cfg.add_symbol(".", Token::Symbol("."));
	cfg.add_symbol("..", Token::Symbol(".."));

	let out = Context::new(input, cfg);
	out
}

fn read_token<'a>(config: &Config, input: &mut Cursor<'a>) -> (LexerResult, Span<'a>) {
	config.read_token(input)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn lexer_should_parse_symbols() {
		let mut ctx = open(&"+ - / *");
		assert_eq!(ctx.token(), Token::Symbol("+"));
		ctx.next();

		assert_eq!(ctx.token(), Token::Symbol("-"));
		ctx.next();

		assert_eq!(ctx.token(), Token::Symbol("/"));
		ctx.next();

		assert_eq!(ctx.token(), Token::Symbol("*"));
		ctx.next();
	}

	#[test]
	fn lexer_should_configure_symbols() {
		let mut ctx = open(&"+ - /// *** ^^^");
		assert_eq!(ctx.token(), Token::Symbol("+"));
		ctx.next();

		assert_eq!(ctx.token(), Token::Symbol("-"));
		ctx.next();

		ctx.add_symbol("///", Token::Symbol("div"));
		ctx.add_symbol("***", Token::Symbol("pow"));
		ctx.add_symbol("^^^", Token::Symbol("car"));

		assert_eq!(ctx.token(), Token::Symbol("div"));
		ctx.next();

		assert_eq!(ctx.token(), Token::Symbol("pow"));
		ctx.next();

		assert_eq!(ctx.token(), Token::Symbol("car"));
		ctx.next();
	}

	#[test]
	fn lexer_should_save_and_restore_configuration() {
		let mut ctx = open(&"//////.");

		// read some symbols before changing the configuration to make sure
		// it doesn't apply retroactively
		assert_eq!(ctx.token(), Token::Symbol("/"));
		ctx.next();
		assert_eq!(ctx.token(), Token::Symbol("/"));
		ctx.next();

		// save the context before customizing symbols
		let mut saved1 = ctx.clone();
		let mut saved2 = ctx.clone();

		// add a custom symbol to the original context
		ctx.add_symbol("//", Token::Symbol("div"));

		// make sure the symbol is applied going forward from the next token
		assert_eq!(ctx.token(), Token::Symbol("div"));
		ctx.next();
		assert_eq!(ctx.token(), Token::Symbol("div"));
		ctx.next();

		// check the end of input
		assert_eq!(ctx.token(), Token::Symbol("."));
		ctx.next();
		assert_eq!(ctx.token(), Token::None);

		// the original clone should have no concept of the new symbol
		assert_eq!(saved1.token(), Token::Symbol("/"));

		// customize another clone of the clone
		let mut other = saved1.clone();
		other.add_symbol("//", Token::Symbol("other_div"));

		// make sure the other clone can be customized
		assert_eq!(other.token(), Token::Symbol("other_div"));
		other.next();
		assert_eq!(other.token(), Token::Symbol("other_div"));
		other.next();
		assert_eq!(other.token(), Token::Symbol("."));
		other.next();
		assert_eq!(other.token(), Token::None);

		// again, the original copy should have no concept of any customization
		assert_eq!(saved1.token(), Token::Symbol("/"));
		saved1.next();
		assert_eq!(saved1.token(), Token::Symbol("/"));
		saved1.next();
		assert_eq!(saved1.token(), Token::Symbol("/"));
		saved1.next();
		assert_eq!(saved1.token(), Token::Symbol("/"));
		saved1.next();
		assert_eq!(saved1.token(), Token::Symbol("."));
		saved1.next();
		assert_eq!(saved1.token(), Token::None);

		// add multiple symbols in a row to exercise any single-owner code path
		saved2.add_symbol("/", Token::Symbol("div1"));
		saved2.add_symbol("//", Token::Symbol("div2"));
		saved2.add_symbol("///", Token::Symbol("div3"));

		assert_eq!(saved2.token(), Token::Symbol("div3"));
		saved2.next();
		assert_eq!(saved2.token(), Token::Symbol("div1")); // also tests overwriting symbols
		saved2.next();
		assert_eq!(saved2.token(), Token::Symbol("."));
		saved2.next();
		assert_eq!(saved2.token(), Token::None);
	}
}
