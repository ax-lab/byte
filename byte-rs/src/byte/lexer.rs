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

	fn tokenize(input: &str) -> (Vec<NodeValue>, Compiler) {
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

	fn word(compiler: &Compiler, name: &str) -> NodeValue {
		NodeValue::from(Token::Word(compiler.get_name(name)))
	}

	fn sym(compiler: &Compiler, name: &str) -> NodeValue {
		NodeValue::from(Token::Symbol(compiler.get_name(name)))
	}

	fn eol() -> NodeValue {
		NodeValue::from(LineBreak)
	}

	fn indent(width: usize) -> NodeValue {
		NodeValue::from(Token::Indent(width))
	}

	fn comment() -> NodeValue {
		NodeValue::from(Comment)
	}

	fn literal(str: &str) -> NodeValue {
		NodeValue::from(Literal(str.to_string()))
	}

	fn int(value: u128) -> NodeValue {
		NodeValue::from(Integer(value))
	}
}
