use super::*;

pub mod chars;
pub mod input;
pub mod scanner;
pub mod symbols;

pub use chars::*;
pub use input::*;
pub use scanner::*;
pub use symbols::*;

pub type Name = Handle<str>;

impl Compiler {
	pub fn get_name<T: AsRef<str>>(&self, name: T) -> Name {
		self.intern(name)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn empty() {
		let (actual, _) = tokenize("");
		assert_eq!(actual, Vec::new());

		let (actual, _) = tokenize("    ");
		assert_eq!(actual, Vec::new());

		let (actual, _) = tokenize("\n");
		assert_eq!(actual, Vec::new());

		let (actual, _) = tokenize("\r");
		assert_eq!(actual, Vec::new());

		let (actual, _) = tokenize("\r\n");
		assert_eq!(actual, Vec::new());

		let (actual, _) = tokenize("\n    ");
		assert_eq!(actual, Vec::new());

		let (actual, _) = tokenize("    \n");
		assert_eq!(actual, Vec::new());

		let (actual, _) = tokenize("\t\n\n\r\n    \r    \n");
		assert_eq!(actual, Vec::new());
	}

	#[test]
	fn simple_scanning() {
		let (actual, compiler) = tokenize("a, b (\n\tsome_name123\n)");
		assert_eq!(
			actual,
			vec![
				word(&compiler, "a"),
				sym(&compiler, ","),
				word(&compiler, "b"),
				sym(&compiler, "("),
				eol(),
				indent(4),
				word(&compiler, "some_name123"),
				eol(),
				sym(&compiler, ")")
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
		let (actual, compiler) = tokenize(input.join("\n").as_str());
		assert_eq!(
			actual,
			vec![
				comment(),
				eol(),
				word(&compiler, "print"),
				literal("hello world!"),
				eol(),
				comment(),
				eol(),
				word(&compiler, "print"),
				int(1),
				sym(&compiler, ","),
				int(2),
				sym(&compiler, ","),
				int(3),
			]
		)
	}

	//----------------------------------------------------------------------------------------------------------------//
	// Helpers
	//----------------------------------------------------------------------------------------------------------------//

	fn tokenize(input: &str) -> (Vec<Node>, Compiler) {
		let compiler = Compiler::new();
		let mut scanner = Scanner::new(compiler.get_ref());
		scanner.register_common_symbols();
		scanner.add_matcher(CommentMatcher);
		scanner.add_matcher(LiteralMatcher);
		scanner.add_matcher(IntegerMatcher);

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
		(output, compiler)
	}

	fn word(compiler: &Compiler, name: &str) -> Node {
		Node::from(Token::Word(compiler.get_name(name)), None)
	}

	fn sym(compiler: &Compiler, name: &str) -> Node {
		Node::from(Token::Symbol(compiler.get_name(name)), None)
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
