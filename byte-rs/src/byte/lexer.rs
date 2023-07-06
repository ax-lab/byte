use super::*;

pub mod chars;
pub mod comment;
pub mod literal;
pub mod number;
pub mod scanner;
pub mod symbols;

pub use chars::*;
pub use comment::*;
pub use literal::*;
pub use number::*;
pub use scanner::*;
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
		let mut get = || actual.next().unwrap();
		check!(get(), Node::Word(s, ..)   if s == "a");
		check!(get(), Node::Symbol(s, ..) if s == ",");
		check!(get(), Node::Word(s, ..)   if s == "b");
		check!(get(), Node::Symbol(s, ..) if s == "(");
		check!(get(), Node::Break(..));
		check!(get(), Node::Indent(4, ..));
		check!(get(), Node::Word(s, ..)   if s == "some_name123");
		check!(get(), Node::Break(..));
		check!(get(), Node::Symbol(s, ..) if s == ")");
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
		let mut get = || actual.next().unwrap();

		check!(get(), Node::Comment(..));
		check!(get(), Node::Break(..));
		check!(get(), Node::Word(s, ..)   if s == "print");
		check!(get(), Node::Literal(s, ..) if s == "hello world!");
		check!(get(), Node::Break(..));
		check!(get(), Node::Comment(..));
		check!(get(), Node::Break(..));
		check!(get(), Node::Word(s, ..)   if s == "print");
		check!(get(), Node::Integer(1, ..));
		check!(get(), Node::Symbol(s, ..) if s == ",");
		check!(get(), Node::Integer(2, ..));
		check!(get(), Node::Symbol(s, ..) if s == ",");
		check!(get(), Node::Integer(3, ..));

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
		let mut scanner = Scanner::new();
		scanner.register_common_symbols();
		scanner.add_matcher(CommentMatcher);
		scanner.add_matcher(LiteralMatcher);
		scanner.add_matcher(IntegerMatcher);

		let context = Context::get();
		let input = context.load_source_text("test", input);
		let mut cursor = input.span();
		let mut errors = Errors::new();
		let mut output = Vec::new();
		while let Some(node) = scanner.scan(&mut cursor, &mut errors) {
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
