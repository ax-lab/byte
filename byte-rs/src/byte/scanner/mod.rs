//! Scanning processes the raw source files or input text and returns a
//! list of [`Expr::Token`] nodes for parsing.
//!
//! This process includes the lexical analysis and tokenization, but includes
//! additional steps such as parsing brackets, lines, and indentation.
//!
//! In essence, the scanning process is responsible for breaking up input
//! sources into their broad structure.
//!
//! The scanner operates like a pipeline. Each step of the pipeline receives
//! a segment of data and generates a list of segments to the next step.
//!
//! The individual scanning steps can be customized, but they always start
//! with a single [`Span`] and end with a [`NodeList`] as a result.
//!

use super::*;

pub mod chars;
pub mod matcher;
pub mod scanning;
pub mod token;

pub use chars::*;
pub use matcher::*;
pub use scanning::*;
pub use token::*;

pub mod match_comment;
pub mod match_literal;
pub mod match_number;
pub mod match_symbols;

pub use match_comment::*;
pub use match_literal::*;
pub use match_number::*;
pub use match_symbols::*;

/// Trait for a [`Token`] matcher used by the [`Matcher`].
pub trait IsMatcher {
	fn try_match(&self, cursor: &mut Span, errors: &mut Errors) -> Option<(Token, Span)>;
}

//====================================================================================================================//
// Tests
//====================================================================================================================//

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn empty() {
		let actual = tokenize("");
		assert!(actual.len() == 0);

		let actual = tokenize("    ");
		assert!(actual.len() == 0);

		let actual = tokenize("\n");
		assert!(actual.len() == 0);

		let actual = tokenize("\r");
		assert!(actual.len() == 0);

		let actual = tokenize("\r\n");
		assert!(actual.len() == 0);

		let actual = tokenize("\n    ");
		assert!(actual.len() == 0);

		let actual = tokenize("    \n");
		assert!(actual.len() == 0);

		let actual = tokenize("\t\n\n\r\n    \r    \n");
		assert!(actual.len() == 0);
	}

	#[test]
	fn simple_scanning() {
		let actual = tokenize("a, b (\n\tsome_name123\n)");
		let mut actual = actual.into_iter();
		let mut get = || actual.next().unwrap().clone();
		check!(get(), Token::Word(s)   if s == "a");
		check!(get(), Token::Symbol(s) if s == ",");
		check!(get(), Token::Word(s)   if s == "b");
		check!(get(), Token::Symbol(s) if s == "(");
		check!(get(), Token::Break(4));
		check!(get(), Token::Word(s)   if s == "some_name123");
		check!(get(), Token::Break(0));
		check!(get(), Token::Symbol(s) if s == ")");
		assert!(actual.next().is_none());
	}

	#[test]
	fn with_matchers() {
		let input = vec![
			"# some single line comment",
			"print 'hello world!'",
			"",
			"#(",
			"    this is a multiline comment (with nested parenthesis)",
			")",
			"",
			"print 1, 2, 3",
		];

		let actual = tokenize(input.join("\n").as_str());
		let mut actual = actual.into_iter();
		let mut get = || actual.next().unwrap().clone();

		check!(get(), Token::Comment);
		check!(get(), Token::Break(0));
		check!(get(), Token::Word(s)   if s == "print");
		check!(get(), Token::Literal(s) if s.as_str() == "hello world!");
		check!(get(), Token::Break(0));
		check!(get(), Token::Comment);
		check!(get(), Token::Break(0));
		check!(get(), Token::Word(s)   if s == "print");
		check!(get(), Token::Integer(1));
		check!(get(), Token::Symbol(s) if s == ",");
		check!(get(), Token::Integer(2));
		check!(get(), Token::Symbol(s) if s == ",");
		check!(get(), Token::Integer(3));

		assert!(actual.next().is_none());
	}

	#[test]
	fn line_and_comments() {
		let input = vec![
			"",
			"",
			"# comment 1",
			"",
			"    ",
			"a",
			"",
			"    ",
			"#(",
			"    comment 2",
			")b",
			"",
			"\t",
			"c",
			"",
			"# comment 3",
		];

		let actual = tokenize(input.join("\n").as_str());
		let mut actual = actual.into_iter();
		let mut get = || actual.next().unwrap().clone();

		check!(get(), Token::Comment);
		check!(get(), Token::Break(0));
		check!(get(), Token::Word(..));
		check!(get(), Token::Break(0));
		check!(get(), Token::Comment);
		check!(get(), Token::Word(..));
		check!(get(), Token::Break(0));
		check!(get(), Token::Word(..));
		check!(get(), Token::Break(0));
		check!(get(), Token::Comment);
	}

	#[test]
	fn line_indent() {
		let input = vec![
			"a", "b", "", "  #", "  c", "", "    d", "", "    ", "    e", "", " f", "    ",
		];
		let actual = tokenize(input.join("\n").as_str());
		let mut actual = actual.into_iter();
		let mut get = || actual.next().unwrap().clone();

		check!(get(), Token::Word(w) if w == "a");
		check!(get(), Token::Break(0));
		check!(get(), Token::Word(w) if w == "b");
		check!(get(), Token::Break(2));
		check!(get(), Token::Comment);
		check!(get(), Token::Break(2));
		check!(get(), Token::Word(w) if w == "c");
		check!(get(), Token::Break(4));
		check!(get(), Token::Word(w) if w == "d");
		check!(get(), Token::Break(4));
		check!(get(), Token::Word(w) if w == "e");
		check!(get(), Token::Break(1));
		check!(get(), Token::Word(w) if w == "f");
		check!(get(), Token::Break(0));
	}

	//----------------------------------------------------------------------------------------------------------------//
	// Helpers
	//----------------------------------------------------------------------------------------------------------------//

	mod macros {
		#[macro_export]
		macro_rules! check {
			($x:expr, $y:pat if $($rest:tt)*) => {
				let x = $x;
				let e = format!("{x:?}");
				assert!(matches!(x, $y if $($rest)*), "match failed: {e}");
			};

			($x:expr, $y:pat) => {
				let x = $x;
				let e = format!("{x:?}");
				assert!(matches!(x, $y), "match failed: it was `{e}`");
			};
		}
	}

	fn tokenize(input: &str) -> Vec<Token> {
		let mut matcher = Matcher::new();
		matcher.register_common_symbols();
		matcher.add_matcher(CommentMatcher);
		matcher.add_matcher(LiteralMatcher);
		matcher.add_matcher(IntegerMatcher);

		let context = Context::get();
		let input = context.load_source_text("test", input);
		let mut cursor = input.span();
		let mut errors = Errors::new();
		let mut output = Vec::new();
		while let Some((token, ..)) = matcher.scan(&mut cursor, &mut errors) {
			output.push(token);
		}

		if errors.len() > 0 {
			eprintln!("\n{errors}");
			panic!("Scanning generated errors");
		}

		assert!(cursor.at_end());
		output
	}
}
