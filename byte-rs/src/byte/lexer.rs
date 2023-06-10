use super::*;

pub mod chars;
pub mod input;
pub mod scanner;
pub mod symbols;

pub use chars::*;
pub use input::*;
pub use scanner::*;
pub use symbols::*;

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn empty() {
		assert_eq!(tokenize(""), Vec::new());
		assert_eq!(tokenize("    "), Vec::new());
		assert_eq!(tokenize("\n"), Vec::new());
		assert_eq!(tokenize("\r"), Vec::new());
		assert_eq!(tokenize("\r\n"), Vec::new());
		assert_eq!(tokenize("\n    "), Vec::new());
		assert_eq!(tokenize("    \n"), Vec::new());
		assert_eq!(tokenize("\t\n\n\r\n    \r    \n"), Vec::new());
	}

	#[test]
	fn simple_scanning() {
		let actual = tokenize("a, b (\n\tsome_name123\n)");
		assert_eq!(
			actual,
			vec![
				word("a"),
				sym(","),
				word("b"),
				sym("("),
				eol(),
				indent(4),
				word("some_name123"),
				eol(),
				sym(")")
			]
		);
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
		assert_eq!(
			actual,
			vec![
				comment(),
				eol(),
				word("print"),
				literal("hello world!"),
				eol(),
				comment(),
				eol(),
				word("print"),
				int(1),
				sym(","),
				int(2),
				sym(","),
				int(3),
			]
		)
	}

	//----------------------------------------------------------------------------------------------------------------//
	// Helpers
	//----------------------------------------------------------------------------------------------------------------//

	fn tokenize(input: &str) -> Vec<Node> {
		let mut scanner = Scanner::with_common_symbols();
		scanner.add_matcher(Arc::new(CommentMatcher));
		scanner.add_matcher(Arc::new(LiteralMatcher));
		scanner.add_matcher(Arc::new(IntegerMatcher));

		let input = Input::new("test", input.as_bytes().to_vec());
		let mut cursor = input.start();
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

	fn word(name: &str) -> Node {
		Node::from(Token::Word(name.into()), None)
	}

	fn sym(name: &str) -> Node {
		Node::from(Token::Symbol(name.into()), None)
	}

	fn eol() -> Node {
		Node::from(Token::Break, None)
	}

	fn indent(width: usize) -> Node {
		Node::from(Token::Indent(width), None)
	}

	fn comment() -> Node {
		Node::from(Comment, None)
	}

	fn literal(str: &str) -> Node {
		Node::from(Literal(str.to_string()), None)
	}

	fn int(value: u128) -> Node {
		Node::from(Integer(value), None)
	}
}
