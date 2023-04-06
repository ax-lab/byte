mod error;
mod lexer;
mod matcher;
mod scanner;
mod stream;
mod token;

pub use error::*;
pub use lexer::*;
pub use matcher::*;
pub use scanner::*;
pub use stream::*;
pub use token::*;

mod comment;
mod indent;
mod symbol;

use comment::*;
use indent::*;
use symbol::*;

/// Creates a new pre-configured [`Lexer`].
pub fn open(input: crate::core::input::Input) -> Lexer {
	use crate::lang::*;

	let mut lexer = Lexer::new(input.start(), Scanner::new());
	lexer.config(|scanner| {
		scanner.add_matcher(Comment);
		scanner.add_matcher(Identifier);
		scanner.add_matcher(Literal);
		scanner.add_matcher(Integer);

		scanner.add_symbol(",", Token::Symbol(","));
		scanner.add_symbol(";", Token::Symbol(";"));
		scanner.add_symbol("++", Token::Symbol("++"));
		scanner.add_symbol("--", Token::Symbol("--"));
		scanner.add_symbol("+", Token::Symbol("+"));
		scanner.add_symbol("-", Token::Symbol("-"));
		scanner.add_symbol("*", Token::Symbol("*"));
		scanner.add_symbol("/", Token::Symbol("/"));
		scanner.add_symbol("%", Token::Symbol("%"));
		scanner.add_symbol("=", Token::Symbol("="));
		scanner.add_symbol("==", Token::Symbol("=="));
		scanner.add_symbol("!", Token::Symbol("!"));
		scanner.add_symbol("?", Token::Symbol("?"));
		scanner.add_symbol(":", Token::Symbol(":"));
		scanner.add_symbol("(", Token::Symbol("("));
		scanner.add_symbol(")", Token::Symbol(")"));
		scanner.add_symbol(".", Token::Symbol("."));
		scanner.add_symbol("..", Token::Symbol(".."));
	});
	lexer
}

#[cfg(test)]
mod tests {
	use crate::core::input::*;

	use super::*;

	fn s(symbol: &'static str) -> Token {
		Token::Symbol(symbol)
	}

	#[test]
	fn empty() {
		test("", &vec![]);
	}

	#[test]
	fn simple() {
		test("+-", &vec![s("+"), s("-")])
	}

	#[test]
	fn skip_spaces() {
		let expected = &vec![s("+"), s("-")];
		test("+ -", expected);
		test("+\t-", expected);
		test("+ \t \t -", expected);
		test("\n+ -", expected);
		test("  \n\n+ -", expected);
		test("\r+ -", expected);
		test("\r\n+ -", expected);
	}

	#[test]
	fn line_breaks() {
		let expected = &vec![s("+"), Token::Break, s("-")];
		test("+\n-", expected);
		test("+\r-", expected);
		test("+\r\n-", expected);
		test("+\r\n\r\n-", expected);

		let mut expected = expected.clone();
		expected.push(Token::Break);
		test("+\n\n-\n", &expected);
	}

	#[test]
	fn indent_simple() {
		// plain indent then dedent
		test(
			"+\n\t-\n+",
			&vec![
				s("+"),
				Token::Break,
				Token::Indent,
				s("-"),
				Token::Break,
				Token::Dedent,
				s("+"),
			],
		);

		// dedent at the end of file
		test(
			"+\n\t-\n\t+",
			&vec![
				s("+"),
				Token::Break,
				Token::Indent,
				s("-"),
				Token::Break,
				s("+"),
				Token::Dedent,
			],
		);

		// multiple indents
		test(
			"+\n\t-\n\t+\n\t\t*\n\t\t/\n\t-",
			&vec![
				s("+"),
				Token::Break,
				Token::Indent,
				s("-"),
				Token::Break,
				s("+"),
				Token::Break,
				Token::Indent,
				s("*"),
				Token::Break,
				s("/"),
				Token::Break,
				Token::Dedent,
				s("-"),
				Token::Dedent,
			],
		);
	}

	fn test(input: &str, expected: &Vec<Token>) {
		let input = Input::open_str("test", input);

		let mut scanner = Scanner::new();
		scanner.add_symbol("+", s("+"));
		scanner.add_symbol("-", s("-"));
		scanner.add_symbol("*", s("*"));
		scanner.add_symbol("/", s("/"));

		let mut output = Vec::new();
		let mut lexer = Lexer::new(input.start(), scanner);
		let mut clone = lexer.clone();
		loop {
			let TokenAt(span, token) = lexer.read();
			if token == Token::None {
				break;
			}

			// sanity check clone
			let TokenAt(clone_span, clone_token) = clone.read();
			assert_eq!(clone_span, span);
			assert_eq!(clone_token, token);

			output.push(token);
		}

		let errors = lexer.errors().list();
		if errors.len() > 0 {
			for it in errors.into_iter() {
				eprintln!("\n{it}");
			}
			eprintln!();
			panic!("lexer generated errors");
		}

		assert_eq!(&output, expected);
	}
}
