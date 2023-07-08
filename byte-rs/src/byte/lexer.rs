use super::*;

pub mod chars;
pub mod comment;
pub mod literal;
pub mod matcher;
pub mod number;
pub mod symbols;

pub use chars::*;
pub use comment::*;
pub use literal::*;
pub use matcher::*;
pub use number::*;
pub use symbols::*;

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
		let mut get = || actual.next().unwrap().bit().clone();
		check!(get(), Bit::Token(Token::Word(s))   if s == "a");
		check!(get(), Bit::Token(Token::Symbol(s)) if s == ",");
		check!(get(), Bit::Token(Token::Word(s))   if s == "b");
		check!(get(), Bit::Token(Token::Symbol(s)) if s == "(");
		check!(get(), Bit::Token(Token::Break));
		check!(get(), Bit::Token(Token::Indent(4)));
		check!(get(), Bit::Token(Token::Word(s))   if s == "some_name123");
		check!(get(), Bit::Token(Token::Break));
		check!(get(), Bit::Token(Token::Symbol(s)) if s == ")");
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
		let mut get = || actual.next().unwrap().bit().clone();

		check!(get(), Bit::Token(Token::Comment));
		check!(get(), Bit::Token(Token::Break));
		check!(get(), Bit::Token(Token::Word(s))   if s == "print");
		check!(get(), Bit::Token(Token::Literal(s)) if s.as_str() == "hello world!");
		check!(get(), Bit::Token(Token::Break));
		check!(get(), Bit::Token(Token::Comment));
		check!(get(), Bit::Token(Token::Break));
		check!(get(), Bit::Token(Token::Word(s))   if s == "print");
		check!(get(), Bit::Token(Token::Integer(1)));
		check!(get(), Bit::Token(Token::Symbol(s)) if s == ",");
		check!(get(), Bit::Token(Token::Integer(2)));
		check!(get(), Bit::Token(Token::Symbol(s)) if s == ",");
		check!(get(), Bit::Token(Token::Integer(3)));

		assert!(actual.next().is_none());
	}

	//----------------------------------------------------------------------------------------------------------------//
	// Helpers
	//----------------------------------------------------------------------------------------------------------------//

	mod macros {
		#[macro_export]
		macro_rules! check {
			($x:expr, $y:pat if $($rest:tt)*) => {
				let x = $x;
				let e = format!("{x}");
				assert!(matches!(x, $y if $($rest)*), "match failed: {e}");
			};

			($x:expr, $y:pat) => {
				let x = $x;
				let e = format!("{x}");
				assert!(matches!(x, $y), "match failed: {e}");
			};
		}
	}

	fn tokenize(input: &str) -> Vec<Node> {
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
		while let Some(node) = matcher.scan(&mut cursor, &mut errors) {
			output.push(node);
		}

		if errors.len() > 0 {
			eprintln!("\n{errors}");
			panic!("Scanning generated errors");
		}

		assert!(cursor.at_end());
		output
	}
}
